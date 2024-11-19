//! mcpatch2管理端第二版
use std::path::PathBuf;

use shared::utility::is_running_under_cargo;
use serde::Deserialize;

use crate::web::serve_web;

pub mod utility;
pub mod subcommand;
pub mod diff;
pub mod common;
pub mod upload;
pub mod web;

const CONFIG_TEMPLATE_STRING: &str = include_str!("../config.template.toml");

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
   pub exclude_rules: Vec<String>,
   pub upload_script_template: String,
   pub upload_script_output: String,

   #[serde(default = "default_serve_listen_port")]
   pub serve_listen_port: u16,

   #[serde(default = "default_serve_listen_addr")]
   pub serve_listen_addr: String,

   #[serde(default = "default_serve_tbf_burst")]
   pub serve_tbf_burst: u32,

   #[serde(default = "default_serve_tbf_rate")]
   pub serve_tbf_rate: u32,
}

fn default_serve_listen_port() -> u16 {
    6700
}

fn default_serve_listen_addr() -> String {
    "0.0.0.0".to_owned()
}

fn default_serve_tbf_burst() -> u32 {
    0
}

fn default_serve_tbf_rate() -> u32 {
    0
}

#[derive(Clone)]
pub struct AppContext {
    pub working_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub public_dir: PathBuf,
    pub index_file: PathBuf,
    pub config: AppConfig
}

impl AppContext {
    pub fn new() -> Self {
        let mut working_dir = std::env::current_dir().unwrap();
        
        if is_running_under_cargo() {
            working_dir = working_dir.join("test");
        }

        let workspace_dir = working_dir.join("workspace");
        let public_dir = working_dir.join("public");
        let version_file = working_dir.join("public/index.json");
        let config_file = working_dir.join("config.toml");

        if !config_file.exists() {
            std::fs::write(&config_file, CONFIG_TEMPLATE_STRING).unwrap();
        }

        let config = toml::from_str::<AppConfig>(&std::fs::read_to_string(&config_file).unwrap()).unwrap();

        AppContext {
            working_dir, 
            workspace_dir, 
            public_dir, 
            index_file: version_file, 
            config,
        }
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let ctx = AppContext::new();

    // 启动web服务器
    serve_web(ctx);
}