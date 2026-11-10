use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::mpsc,
    time::{Duration, Instant},
};
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub repository: String,
    pub automatic: bool,
}
impl Default for Settings {
    fn default() -> Self {
        let value: serde_json::Value =
            serde_json::from_str(include_str!("../release-config.json")).unwrap_or_default();
        Self {
            repository: value["repository"].as_str().unwrap_or("").into(),
            automatic: true,
        }
    }
}
pub fn repository(input: &str) -> Result<String, String> {
    let s = input.trim().trim_end_matches('/');
    let s = s.strip_prefix("https://github.com/").unwrap_or(s);
    let s = s
        .strip_suffix("/releases/latest")
        .or_else(|| s.strip_suffix("/releases"))
        .unwrap_or(s);
    let parts: Vec<_> = s.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|p| {
            p.is_empty()
                || p.len() > 100
                || p.starts_with('.')
                || !p
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
        })
    {
        return Err("Informe conta/repositorio ou o link HTTPS do repositório no GitHub.".into());
    }
    Ok(s.into())
}
fn version(s: &str) -> Option<[u64; 3]> {
    let parts: Vec<_> = s.strip_prefix('v').unwrap_or(s).split('.').collect();
    if parts.len() != 3
        || parts.iter().any(|p| {
            p.is_empty()
                || !p.bytes().all(|b| b.is_ascii_digit())
                || (p.len() > 1 && p.starts_with('0'))
        })
    {
        return None;
    }
    Some([
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ])
}
#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub version: String,
    pub url: String,
    pub package: String,
}
#[derive(Deserialize)]
struct ApiRelease {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset {
    name: String,
    state: String,
}
fn parse_release(bytes: &[u8], repo: &str, current: &str) -> Result<Option<Release>, String> {
    let repo = repository(repo)?;
    let r: ApiRelease = serde_json::from_slice(bytes)
        .map_err(|_| "O servidor retornou uma release inválida.".to_string())?;
    if r.draft || r.prerelease {
        return Ok(None);
    }
    let latest =
        version(&r.tag_name).ok_or("A release não usa uma versão estável no formato v1.2.3.")?;
    if latest <= version(current).ok_or("Versão instalada inválida.")? {
        return Ok(None);
    }
    let number = r.tag_name.trim_start_matches('v');
    if !r.assets.iter().any(|a| {
        [
            format!("test-craft-{number}-windows-x64.msi"),
            format!("test-craft-{number}-windows-x64.zip"),
        ]
        .contains(&a.name)
            && a.state == "uploaded"
    }) {
        return Err(format!(
            "A versão {number} existe, mas o pacote Windows ainda não está disponível."
        ));
    }
    let package = format!("test-craft-{number}-windows-x64.msi");
    Ok(Some(Release {
        version: number.into(),
        url: format!("https://github.com/{repo}/releases/download/{}", r.tag_name),
        package,
    }))
}
fn fetch(repo: &str) -> Result<Option<Release>, String> {
    let repo = repository(repo)?;
    let mut command = std::process::Command::new("curl.exe");
    command.args([
        "--fail",
        "--silent",
        "--show-error",
        "--proto",
        "=https",
        "--tlsv1.2",
        "--connect-timeout",
        "8",
        "--max-time",
        "20",
        "--max-filesize",
        "2097152",
        "--user-agent",
        concat!("TestCraft/", env!("CARGO_PKG_VERSION")),
        "--header",
        "Accept: application/vnd.github+json",
        "--header",
        "X-GitHub-Api-Version: 2022-11-28",
        &format!("https://api.github.com/repos/{repo}/releases/latest"),
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let output = command.output().map_err(|_| "Não foi possível consultar atualizações. Verifique se curl.exe está disponível no Windows.".to_string())?;
    if !output.status.success() {
        return Err("Não foi possível consultar releases. Confira a conexão, o repositório público e se há uma release publicada. Tente novamente mais tarde.".into());
    }
    parse_release(&output.stdout, &repo, VERSION)
}
pub struct Updater {
    pub settings: Settings,
    pub message: String,
    pub available: Option<Release>,
    pub offer: bool,
    receiver: Option<mpsc::Receiver<Result<Option<Release>, String>>>,
    last_check: Option<Instant>,
    install_receiver: Option<mpsc::Receiver<Result<(), String>>>,
    pub close_requested: bool,
}
impl Updater {
    pub fn load(data: &Path) -> Self {
        let settings = std::fs::read(data.join("atualizacoes.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Self {
            settings,
            message: "Configure o repositório para receber atualizações.".into(),
            available: None,
            offer: false,
            receiver: None,
            last_check: None,
            install_receiver: None,
            close_requested: false,
        }
    }
    pub fn save(&mut self, data: &Path, input: &str, automatic: bool) -> Result<(), String> {
        let settings = Settings {
            repository: repository(input)?,
            automatic,
        };
        std::fs::create_dir_all(data).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
        std::fs::write(data.join("atualizacoes.json"), bytes).map_err(|e| e.to_string())?;
        self.settings = settings;
        self.available = None;
        self.offer = false;
        self.last_check = None;
        self.message = "Preferências de atualização salvas.".into();
        Ok(())
    }
    pub fn installing(&self) -> bool {
        self.install_receiver.is_some() || self.close_requested
    }
    pub fn busy(&self) -> bool {
        self.receiver.is_some() || self.install_receiver.is_some()
    }
    pub fn check(&mut self, ctx: &eframe::egui::Context) {
        if self.busy() {
            return;
        }
        let repo = match repository(&self.settings.repository) {
            Ok(r) => r,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        self.available = None;
        self.offer = false;
        self.message = "Verificando releases…".into();
        self.last_check = Some(Instant::now());
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(fetch(&repo));
            ctx.request_repaint();
        });
    }
    pub fn install(&mut self, ctx: &eframe::egui::Context) {
        if self.busy() {
            return;
        }
        let Some(release) = self.available.clone() else {
            return;
        };
        self.offer = false;
        self.message =
            "Baixando e verificando atualização… Aguarde a autorização do Windows.".into();
        let (tx, rx) = mpsc::channel();
        self.install_receiver = Some(rx);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(download_and_launch(&release));
            ctx.request_repaint();
        });
    }
    pub fn tick(&mut self, ctx: &eframe::egui::Context) {
        if let Some(rx) = &self.install_receiver {
            match rx.try_recv() {
                Ok(result) => {
                    self.install_receiver = None;
                    match result {
                        Ok(()) => {
                            self.close_requested = true;
                            self.message = "Instalação autorizada. Fechando para atualizar…".into();
                        }
                        Err(e) => self.message = e,
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.install_receiver = None;
                    self.message = "Atualização interrompida. Tente novamente.".into();
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
            ctx.request_repaint_after(Duration::from_millis(200));
        }
        if let Some(rx) = &self.receiver {
            match rx.try_recv() {
                Ok(result) => {
                    self.receiver = None;
                    match result {
                        Ok(Some(r)) => {
                            self.message = format!("A versão {} está disponível.", r.version);
                            self.available = Some(r);
                            self.offer = true;
                        }
                        Ok(None) => {
                            self.message = format!("Você está na versão mais recente ({VERSION}).");
                            self.available = None;
                            self.offer = false;
                        }
                        Err(e) => {
                            self.message = e;
                        }
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.receiver = None;
                    self.message = "A consulta foi interrompida. Tente novamente.".into();
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if self.settings.automatic && !self.settings.repository.trim().is_empty() {
            if self
                .last_check
                .map(|t| t.elapsed() >= INTERVAL)
                .unwrap_or(true)
            {
                self.check(ctx);
            }
            ctx.request_repaint_after(Duration::from_secs(30));
        }
    }
}

fn hidden_command(exe: &std::ffi::OsStr) -> std::process::Command {
    let mut command = std::process::Command::new(exe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
}
fn checksum(bytes: &[u8], name: &str) -> Result<String, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "Checksum inválido.")?;
    let parts = text.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 2
        || parts[1].trim_start_matches('*') != name
        || parts[0].len() != 64
        || !parts[0].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(
            "O checksum do pacote está ausente ou inválido. A instalação foi cancelada.".into(),
        );
    }
    Ok(parts[0].to_ascii_lowercase())
}
fn download_and_launch(release: &Release) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let installed = exe
        .parent()
        .is_some_and(|p| p.join("installed.txt").exists());
    let name = if installed {
        release.package.clone()
    } else {
        release.package.replace(".msi", ".zip")
    };
    let folder = std::env::temp_dir().join(format!(
        "TestCraft-update-{}-{}",
        std::process::id(),
        crate::now()
    ));
    std::fs::create_dir(&folder).map_err(|e| e.to_string())?;
    let package = folder.join(&name);
    let curl = std::path::PathBuf::from(
        std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into()),
    )
    .join("System32/curl.exe");
    let fetch = |url: &str, path: &Path, limit: &str| -> Result<(), String> {
        let result = hidden_command(curl.as_os_str())
            .args([
                "--fail",
                "--silent",
                "--show-error",
                "--location",
                "--proto",
                "=https",
                "--proto-redir",
                "=https",
                "--tlsv1.2",
                "--connect-timeout",
                "15",
                "--max-time",
                "600",
                "--max-filesize",
                limit,
                "--output",
            ])
            .arg(path)
            .arg(url)
            .output()
            .map_err(|e| e.to_string())?;
        if !result.status.success() {
            return Err("Não foi possível baixar o pacote e seu checksum. Confira a conexão e tente novamente.".into());
        }
        Ok(())
    };
    fetch(&format!("{}/{}", release.url, name), &package, "268435456")?;
    let sha = folder.join("package.sha256");
    fetch(&format!("{}/{}.sha256", release.url, name), &sha, "4096")?;
    let hash = checksum(&std::fs::read(sha).map_err(|e| e.to_string())?, &name)?;
    let helper = folder.join("install.ps1");
    std::fs::write(&helper, include_str!("update-install.ps1")).map_err(|e| e.to_string())?;
    let job = folder.join("job.json");
    let result_path = crate::data_root().join("update-result.txt");
    let job_bytes=serde_json::to_vec(&serde_json::json!({"package":package,"sha256":hash,"executable":exe,"installed":installed,"parentId":std::process::id(),"result":result_path})).map_err(|e|e.to_string())?;
    std::fs::write(&job, job_bytes).map_err(|e| e.to_string())?;
    let launcher = folder.join("launch.ps1");
    std::fs::write(&launcher, include_str!("update-launch.ps1")).map_err(|e| e.to_string())?;
    let powershell = std::path::PathBuf::from(
        std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into()),
    )
    .join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let output = hidden_command(powershell.as_os_str())
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(launcher)
        .arg("-Job")
        .arg(job)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "Atualização cancelada ou não autorizada: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn payload(tag: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"tag_name":tag,"assets":[{"name":format!("test-craft-{}-windows-x64.zip",tag.trim_start_matches('v')),"state":"uploaded"}]})).unwrap()
    }
    #[test]
    fn refuses_wrong_or_malformed_checksums() {
        let hash = "a".repeat(64);
        assert_eq!(
            checksum(format!("{hash}  package.msi\r\n").as_bytes(), "package.msi").unwrap(),
            hash
        );
        for text in [
            format!("{hash} wrong.msi"),
            format!("{} package.msi", "z".repeat(64)),
            format!("{hash} package.msi extra"),
            "abc package.msi".into(),
        ] {
            assert!(checksum(text.as_bytes(), "package.msi").is_err());
        }
    }
    #[test]
    fn accepts_only_github_repositories() {
        assert_eq!(
            repository(" https://github.com/escola/test-craft/releases/latest/ ").unwrap(),
            "escola/test-craft"
        );
        for bad in [
            "",
            "https://evil.com/a/b",
            "a/b/c",
            "a/b?cmd=x",
            "a/..",
            "a/b&whoami",
            "a/b#x",
        ] {
            assert!(repository(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn compares_numeric_stable_versions() {
        assert!(parse_release(&payload("v0.10.0"), "a/b", "0.2.0")
            .unwrap()
            .is_some());
        for tag in ["v0.2.0", "v0.1.9"] {
            assert!(parse_release(&payload(tag), "a/b", "0.2.0")
                .unwrap()
                .is_none());
        }
        for tag in ["v1.0.0-beta", "nightly", "v01.0.0"] {
            assert!(parse_release(&payload(tag), "a/b", "0.2.0").is_err());
        }
    }
    #[test]
    fn rejects_unavailable_packages_and_malformed_responses() {
        assert!(parse_release(br#"{"tag_name":"v1.0.0","assets":[]}"#, "a/b", "0.2.0").is_err());
        assert!(parse_release(b"<html>offline</html>", "a/b", "0.2.0").is_err());
        for flag in ["draft", "prerelease"] {
            let mut v: serde_json::Value = serde_json::from_slice(&payload("v1.0.0")).unwrap();
            v[flag] = true.into();
            assert!(
                parse_release(&serde_json::to_vec(&v).unwrap(), "a/b", "0.2.0")
                    .unwrap()
                    .is_none()
            );
        }
    }
    #[test]
    fn uses_trusted_release_url() {
        let r = parse_release(&payload("v1.2.0"), "escola/test-craft", "0.2.0")
            .unwrap()
            .unwrap();
        assert_eq!(
            r.url,
            "https://github.com/escola/test-craft/releases/download/v1.2.0"
        );
    }
    #[test]
    fn offers_msi_releases() {
        let payload = br#"{"tag_name":"v0.3.0","assets":[{"name":"test-craft-0.3.0-windows-x64.msi","state":"uploaded"}]}"#;
        assert!(parse_release(payload, "a/b", "0.2.0").unwrap().is_some());
    }

    #[test]
    fn persists_preferences_and_preserves_them_on_invalid_input() {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/update-preferences-test");
        std::fs::create_dir_all(&folder).unwrap();
        let mut updater = Updater::load(&folder);
        updater
            .save(
                &folder,
                "https://github.com/escola/test-craft/releases",
                false,
            )
            .unwrap();
        let saved = Updater::load(&folder);
        assert_eq!(saved.settings.repository, "escola/test-craft");
        assert!(!saved.settings.automatic);
        assert!(updater
            .save(&folder, "https://evil.com/repo", true)
            .is_err());
        assert_eq!(updater.settings.repository, "escola/test-craft");
        assert!(!Updater::load(&folder).settings.automatic);
    }
    #[test]
    fn offers_updates_from_background_and_keeps_working_after_failure() {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/update-state-test");
        let mut updater = Updater::load(&folder);
        updater.settings.automatic = false;
        let context = eframe::egui::Context::default();
        let (tx, rx) = mpsc::channel();
        updater.receiver = Some(rx);
        tx.send(Ok(Some(Release {
            version: "1.0.0".into(),
            package: "test-craft-1.0.0-windows-x64.msi".into(),
            url: "https://github.com/escola/test-craft/releases/latest".into(),
        })))
        .unwrap();
        updater.tick(&context);
        assert!(updater.offer);
        assert!(!updater.busy());
        assert_eq!(updater.available.as_ref().unwrap().version, "1.0.0");
        let (tx, rx) = mpsc::channel();
        updater.receiver = Some(rx);
        tx.send(Err("Sem conexão".into())).unwrap();
        updater.tick(&context);
        assert_eq!(updater.message, "Sem conexão");
        assert!(!updater.busy());
    }
}
