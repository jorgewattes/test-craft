use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::mpsc,
    time::{Duration, Instant},
};
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
const RETRY_INTERVAL: Duration = Duration::from_secs(60);
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
fn query(url: &str, extra: &[&str]) -> Result<Vec<u8>, String> {
    let output = hidden_command(curl_path().as_os_str())
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
            "8",
            "--max-time",
            "20",
            "--max-filesize",
            "2097152",
            "--user-agent",
            concat!("TestCraft/", env!("CARGO_PKG_VERSION")),
            "--header",
            "Cache-Control: no-cache",
        ])
        .args(extra)
        .arg(url)
        .output()
        .map_err(|_| {
            "Não foi possível iniciar a consulta de atualizações do Windows.".to_string()
        })?;
    if !output.status.success() {
        return Err(
            "Não foi possível consultar releases. Confira a conexão e tente novamente.".into(),
        );
    }
    Ok(output.stdout)
}
fn public_tag(url: &[u8], repo: &str) -> Result<String, String> {
    let prefix = format!("https://github.com/{repo}/releases/tag/");
    let tag = std::str::from_utf8(url)
        .ok()
        .and_then(|s| s.trim().strip_prefix(&prefix))
        .filter(|tag| version(tag).is_some())
        .ok_or("O GitHub não retornou uma release estável deste repositório.")?;
    Ok(tag.into())
}
fn parse_public_release(
    html: &[u8],
    repo: &str,
    tag: &str,
    current: &str,
) -> Result<Option<Release>, String> {
    let repo = repository(repo)?;
    let latest = version(tag).ok_or("Versão publicada inválida.")?;
    if latest <= version(current).ok_or("Versão instalada inválida.")? {
        return Ok(None);
    }
    let html = std::str::from_utf8(html).map_err(|_| "Lista de pacotes inválida.")?;
    let number = tag.trim_start_matches('v');
    let mut assets = Vec::new();
    for extension in ["msi", "msi.sha256", "zip", "zip.sha256"] {
        let name = format!("test-craft-{number}-windows-x64.{extension}");
        let relative = format!("/{repo}/releases/download/{tag}/{name}");
        if ![
            format!("href=\"{relative}\""),
            format!("href=\"https://github.com{relative}\""),
        ]
        .iter()
        .any(|link| html.contains(link))
        {
            return Err(format!(
                "A versão {number} existe, mas seus pacotes e checksums ainda não estão completos."
            ));
        }
        assets.push(serde_json::json!({"name": name, "state":"uploaded"}));
    }
    let payload = serde_json::to_vec(&serde_json::json!({"tag_name":tag,"assets":assets}))
        .map_err(|e| e.to_string())?;
    parse_release(&payload, &repo, current)
}
fn fetch_public(repo: &str) -> Result<Option<Release>, String> {
    // The public release pages keep working when the unauthenticated API is rate-limited.
    let url = query(
        &format!("https://github.com/{repo}/releases/latest"),
        &[
            "--head",
            "--output",
            "NUL",
            "--write-out",
            "%{url_effective}",
        ],
    )?;
    let tag = public_tag(&url, repo)?;
    if version(&tag) <= version(VERSION) {
        return Ok(None);
    }
    let html = query(
        &format!("https://github.com/{repo}/releases/expanded_assets/{tag}"),
        &[],
    )?;
    parse_public_release(&html, repo, &tag, VERSION)
}
fn fetch(repo: &str) -> Result<Option<Release>, String> {
    let repo = repository(repo)?;
    match query(
        &format!("https://api.github.com/repos/{repo}/releases/latest"),
        &[
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
        ],
    )
    .and_then(|bytes| parse_release(&bytes, &repo, VERSION))
    {
        Ok(release) => Ok(release),
        Err(_) => fetch_public(&repo),
    }
}
pub struct Updater {
    pub settings: Settings,
    pub message: String,
    pub available: Option<Release>,
    pub offer: bool,
    pub check_failed: bool,
    receiver: Option<mpsc::Receiver<Result<Option<Release>, String>>>,
    last_check: Option<Instant>,
    install_receiver: Option<mpsc::Receiver<Result<(), String>>>,
    pub close_requested: bool,
}
impl Updater {
    pub fn load(data: &Path) -> Self {
        let mut settings: Settings = std::fs::read(data.join("atualizacoes.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        // Older versions saved an empty repository before GitHub was configured.
        // Recover the bundled repository while preserving the automatic-check choice.
        if settings.repository.trim().is_empty() {
            settings.repository = Settings::default().repository;
        }
        Self {
            settings,
            message: "As atualizações são verificadas ao abrir o Test Craft.".into(),
            available: None,
            offer: false,
            check_failed: false,
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
        self.check_failed = false;
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
        self.check_failed = false;
        self.message = "Verificando atualizações…".into();
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
                    self.last_check = Some(Instant::now());
                    self.check_failed = false;
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
                            self.check_failed = true;
                            self.message = e;
                        }
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.receiver = None;
                    self.check_failed = true;
                    self.message = "A consulta foi interrompida. Tente novamente.".into();
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
            ctx.request_repaint_after(Duration::from_millis(200));
        }
        if self.settings.automatic && !self.settings.repository.trim().is_empty() {
            if self.check_due() {
                self.check(ctx);
            }
            ctx.request_repaint_after(Duration::from_secs(30));
        }
    }
    fn check_due(&self) -> bool {
        self.settings.automatic
            && !self.settings.repository.trim().is_empty()
            && !self.busy()
            && self
                .last_check
                .map(|t| {
                    t.elapsed()
                        >= if self.check_failed {
                            RETRY_INTERVAL
                        } else {
                            INTERVAL
                        }
                })
                .unwrap_or(true)
    }
}

fn curl_path() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:/Windows".into()))
        .join("System32/curl.exe")
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
    let curl = curl_path();
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
    fn recovers_legacy_empty_repository_without_overriding_opt_out() {
        let folder =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/update-legacy-settings-test");
        std::fs::create_dir_all(&folder).unwrap();
        for automatic in [true, false] {
            std::fs::write(
                folder.join("atualizacoes.json"),
                serde_json::to_vec(&serde_json::json!({"repository":"  ","automatic":automatic}))
                    .unwrap(),
            )
            .unwrap();
            let updater = Updater::load(&folder);
            assert_eq!(updater.settings.repository, Settings::default().repository);
            assert_eq!(updater.settings.automatic, automatic);
            assert_eq!(updater.check_due(), automatic);
        }
    }
    #[test]
    fn checks_on_startup_and_retries_failures_without_waiting_six_hours() {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("checks/update-schedule-test");
        let mut updater = Updater::load(&folder);
        assert!(updater.check_due());
        updater.last_check = Some(Instant::now());
        assert!(!updater.check_due());
        updater.last_check = Some(Instant::now() - Duration::from_secs(90));
        assert!(!updater.check_due());
        updater.check_failed = true;
        assert!(updater.check_due());
        let (_tx, rx) = mpsc::channel();
        updater.receiver = Some(rx);
        assert!(!updater.check_due());
        updater.receiver = None;
        updater.settings.automatic = false;
        assert!(!updater.check_due());
    }

    #[test]
    fn public_fallback_detects_updates_and_requires_packages_with_checksums() {
        let repo = "escola/test-craft";
        let tag = public_tag(
            b"https://github.com/escola/test-craft/releases/tag/v1.2.0\n",
            repo,
        )
        .unwrap();
        let html: String = ["msi", "msi.sha256", "zip", "zip.sha256"].iter()
            .map(|ext| format!("<a href=\"/{repo}/releases/download/{tag}/test-craft-1.2.0-windows-x64.{ext}\">Download</a>"))
            .collect();
        let release = parse_public_release(html.as_bytes(), repo, &tag, "1.1.0")
            .unwrap()
            .unwrap();
        assert_eq!(release.version, "1.2.0");
        assert_eq!(
            release.url,
            "https://github.com/escola/test-craft/releases/download/v1.2.0"
        );
        assert!(parse_public_release(html.as_bytes(), repo, &tag, "1.2.0")
            .unwrap()
            .is_none());
        assert!(parse_public_release(
            html.replace(".zip.sha256", ".zip.missing").as_bytes(),
            repo,
            &tag,
            "1.1.0"
        )
        .is_err());
        for url in [
            "https://evil.com/escola/test-craft/releases/tag/v1.2.0",
            "https://github.com/other/repo/releases/tag/v1.2.0",
            "https://github.com/escola/test-craft/releases/tag/v1.2.0-beta",
            "https://github.com/escola/test-craft/releases/tag/v1.2.0/extra",
        ] {
            assert!(public_tag(url.as_bytes(), repo).is_err());
        }
    }

    #[test]
    #[ignore = "Requires public GitHub connectivity"]
    fn public_github_release_check() {
        let repo = Settings::default().repository;
        let url = query(
            &format!("https://github.com/{repo}/releases/latest"),
            &[
                "--head",
                "--output",
                "NUL",
                "--write-out",
                "%{url_effective}",
            ],
        )
        .unwrap();
        let tag = public_tag(&url, &repo).unwrap();
        let html = query(
            &format!("https://github.com/{repo}/releases/expanded_assets/{tag}"),
            &[],
        )
        .unwrap();
        let release = parse_public_release(&html, &repo, &tag, "0.0.0")
            .unwrap()
            .unwrap();
        println!(
            "Public GitHub fallback OK: {} with MSI, ZIP and SHA-256 files",
            release.version
        );
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
        assert!(updater.check_failed);
        assert!(!updater.check_due());
        let (tx, rx) = mpsc::channel();
        updater.receiver = Some(rx);
        tx.send(Ok(None)).unwrap();
        updater.tick(&context);
        assert!(!updater.check_failed);
        assert!(updater.available.is_none());
        assert!(!updater.busy());
    }
}
