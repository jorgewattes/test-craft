use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=assets/test-craft.ico");
    if !env::var("TARGET")
        .unwrap_or_default()
        .contains("windows-msvc")
    {
        return;
    }
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let rc = out.join("icon.rc");
    let res = out.join("icon.res");
    fs::write(
        &rc,
        format!(
            "1 ICON \"{}\"\n",
            root.join("assets/test-craft.ico")
                .display()
                .to_string()
                .replace('\\', "/")
        ),
    )
    .unwrap();
    let mut compiler = None;
    if let Some(programs) = env::var_os("ProgramFiles(x86)") {
        let bin = PathBuf::from(programs).join("Windows Kits/10/bin");
        let mut versions: Vec<_> = fs::read_dir(&bin)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect();
        versions.sort();
        compiler = versions
            .into_iter()
            .rev()
            .map(|p| p.join("x64/rc.exe"))
            .find(|p| p.is_file());
    }
    let compiler = compiler.unwrap_or_else(|| PathBuf::from("rc.exe"));
    let status = Command::new(compiler)
        .arg("/nologo")
        .arg("/fo")
        .arg(&res)
        .arg(&rc)
        .status()
        .expect("Windows SDK Resource Compiler necessário para incorporar o ícone");
    assert!(status.success(), "Falha ao compilar o ícone Windows");
    println!("cargo:rustc-link-arg={}", res.display());
}
