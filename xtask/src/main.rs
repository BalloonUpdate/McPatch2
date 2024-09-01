use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

type ProcessResult = Result<(), Box<dyn std::error::Error>>;

fn main() -> ProcessResult {
    let task = std::env::args().nth(1);

    match task.as_deref() {
        Some("client") => dist_binary("mcpatch-client", "client"),
        Some("manager") => dist_binary("mcpatch-manager", "manager"),
        _ => print_help(),
    }
}

fn dist_binary(crate_name: &str, production_name: &str) -> ProcessResult {
    std::env::set_var("RUST_BACKTRACE", "1");

    let ref_name = github_ref_name();
    let dist_dir = project_root().join("target/dist");
    let target = TargetInfo::get(crate_name, production_name, &ref_name, &dist_dir);

    // build artifacts
    let cargo = std::env::var("CARGO").unwrap();

    let mut cmd = Command::new(cargo);
    cmd.current_dir(project_root());
    cmd.args(&["build", "--release", "--bin", crate_name, "--target", &target.rustc_target]);
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