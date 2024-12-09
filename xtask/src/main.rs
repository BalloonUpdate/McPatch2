use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use zip::ZipArchive;

type ProcessResult = Result<(), Box<dyn std::error::Error>>;

fn main() -> ProcessResult {
    let mut args = std::env::args();
    let task = args.nth(1);

    match task.as_deref() {
        Some("client") => dist_client(),
        Some("manager") => dist_manager(),
        _ => print_help(),
    }
}

fn dist_client() -> ProcessResult {
    dist_binary("client", "c", None)
}

fn dist_manager() -> ProcessResult {
    let version_label = &std::env::var("GITHUB_REF_NAME").unwrap()[1..];

    let dist_url = format!("https://github.com/BalloonUpdate/McPatch2Web/releases/download/v{}/dist.zip", version_label);
    let response = reqwest::blocking::get(dist_url).unwrap();

    std::fs::write("webpage.zip", response.bytes().unwrap()).unwrap();

    std::fs::create_dir_all("test/webpage").unwrap();

    unzip_file("webpage.zip", "test/webpage");

    dist_binary("manager", "m", Some("bundle-webpage".to_owned()))
}

fn unzip_file(zip_path: &str, output_dir: &str) {
    let file = std::fs::File::open(zip_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = std::path::Path::new(output_dir).join(file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p).unwrap();
            }
            let mut outfile = std::fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }
}

fn dist_binary(crate_name: &str, production_name: &str, features: Option<String>) -> ProcessResult {
    std::env::set_var("RUST_BACKTRACE", "1");

    let ref_name = github_ref_name();
    let dist_dir = project_root().join("target/dist");
    let target = TargetInfo::get(crate_name, production_name, &ref_name, &dist_dir);

    // build artifacts
    let cargo = std::env::var("CARGO").unwrap();

    let mut cmd = Command::new(cargo);

    cmd.current_dir(project_root());

    let mut args = Vec::<String>::new();

    args.push("build".to_owned());
    args.push("--release".to_owned());
    args.push("--bin".to_owned());
    args.push(crate_name.to_owned());
    args.push("--target".to_owned());
    args.push(target.rustc_target.to_owned());

    if let Some(features) = features {
        args.push("--features".to_owned());
        args.push(features.to_owned());
    }

    cmd.args(args);

    let status = cmd.status()?;

    if !status.success() {
        Err("cargo build failed")?;
    }

    // pick up artifacts
    drop(std::fs::remove_dir_all(&dist_dir));
    std::fs::create_dir_all(&dist_dir).unwrap();

    // executable
    std::fs::copy(&target.artifact_path, dist_dir.join(&target.artifact_path_versioned)).unwrap();

    // symbol
    if let Some(symbols) = target.symbols_path {
        std::fs::copy(&symbols, dist_dir.join(&target.symbols_path_versioned.unwrap())).unwrap();
    }

    Ok(())
}

fn print_help() -> ProcessResult {
    return Err("use 'client' or 'manager'")?;
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn github_ref_name() -> String {
    std::env::var("GITHUB_REF_NAME").map(|e| e[1..].to_owned()).unwrap_or("0.0.0".to_owned())
}

struct TargetInfo {
    rustc_target: String,

    artifact_path: PathBuf,
    symbols_path: Option<PathBuf>,

    artifact_path_versioned: PathBuf,
    symbols_path_versioned: Option<PathBuf>,
}

impl TargetInfo {
    fn get(crate_name: &str, production_name: &str, version_label: &str, dist_dir: &Path) -> Self {
        let rustc_target = match std::env::var("MP_RUSTC_TARGET") {
            Ok(t) => t,
            Err(_) => {
                if cfg!(target_os = "linux") {
                    "x86_64-unknown-linux-gnu".to_owned()
                } else if cfg!(target_os = "windows") {
                    "x86_64-pc-windows-msvc".to_owned()
                } else if cfg!(target_os = "macos") {
                    "x86_64-apple-darwin".to_owned()
                } else {
                    panic!("Unsupported OS, maybe try setting MP_RUSTC_TARGET")
                }
            },
        };
        let profile_path = project_root().join(format!("target/{}/release", &rustc_target));
        let is_windows = rustc_target.contains("-windows-");

        let (exe_suffix, symbols_suffix) = match is_windows {
            true => (".exe", Some(".pdb")),
            false => ("", None),
        };

        let symbols_name = symbols_suffix.map(|e| format!("{}{e}", crate_name.replace("-", "_")));
        let symbols_name_versioned = symbols_suffix.map(|e| format!("{production_name}-{version_label}-{rustc_target}{e}"));

        let artifact_name = format!("{crate_name}{exe_suffix}");
        let artifact_name_versioned = format!("{production_name}-{version_label}-{rustc_target}{exe_suffix}");

        let artifact_path = profile_path.join(&artifact_name);
        let symbols_path = symbols_name.as_ref().map(|e| profile_path.join(e));
        
        let artifact_path_versioned = dist_dir.join(&artifact_name_versioned);
        let symbols_path_versioned = symbols_name_versioned.as_ref().map(|e| dist_dir.join(e));

        Self { rustc_target, artifact_path, symbols_path, artifact_path_versioned, symbols_path_versioned }
    }
}