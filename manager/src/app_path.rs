use std::path::PathBuf;

use shared::utility::is_running_under_cargo;

#[derive(Clone)]
pub struct AppPath {
    pub working_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub public_dir: PathBuf,
    pub index_file: PathBuf,
    pub config_file: PathBuf,
    pub auth_file: PathBuf,
}

impl AppPath {
    pub fn new() -> Self {
        let mut working_dir = std::env::current_dir().unwrap();
        
        if is_running_under_cargo() {
            working_dir = working_dir.join("test");
        }

        let workspace_dir = working_dir.join("workspace");
        let public_dir = working_dir.join("public");
        let index_file = working_dir.join("public/index.json");
        let config_file = working_dir.join("config.toml");
        let auth_file = working_dir.join("user.toml");

        std::fs::create_dir_all(&workspace_dir).unwrap();
        std::fs::create_dir_all(&public_dir).unwrap();

        Self {
            working_dir,
            workspace_dir,
            public_dir,
            index_file,
            config_file,
            auth_file,
        }
    }
}
