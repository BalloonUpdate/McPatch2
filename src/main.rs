use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use serde::Deserialize;

use crate::subcommand::check::do_check;
use crate::subcommand::combine::do_combine;
use crate::subcommand::pack::do_pack;
use crate::subcommand::test::do_test;
use crate::utility::is_running_under_cargo;

pub mod diff;
pub mod utility;
pub mod data;
pub mod subcommand;
pub mod common;

const CONFIG_TEMPLATE_STRING: &str = include_str!("../config.template.toml");

#[derive(Parser)]
struct CommandLineInterface {
    // #[arg(long, action = clap::ArgAction::Count)]
    // json_mode: u8,

    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    /// 打包一个新的版本
    Pack {
        /// 指定新的版本号
        version_label: String
    },

    /// 检查工作空间的文件修改情况
    Check,

    /// 合并更新包
    Combine,
    
    /// 测试所有更新包是否能正常读取
    Test,
}

#[derive(Deserialize)]
pub struct AppConfig {
   filter_rules: Vec<String>,
}

pub struct AppContext {
    pub working_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub public_dir: PathBuf,
    pub index_file_official: PathBuf,
    pub index_file_internal: PathBuf,
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
        let version_file_official = working_dir.join("public/index.json");
        let version_file_internal = working_dir.join("public/index.internal.json");
        let config_file = working_dir.join("config.toml");

        if !config_file.exists() {
            std::fs::write(&config_file, CONFIG_TEMPLATE_STRING).unwrap();
        }

        let config = toml::from_str::<AppConfig>(&std::fs::read_to_string(&config_file).unwrap()).unwrap();

        AppContext {
            working_dir, 
            workspace_dir, 
            public_dir, 
            index_file_official: version_file_official, 
            index_file_internal: version_file_internal, 
            config,
        }
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    
    let context = AppContext::new();

    std::fs::create_dir_all(&context.workspace_dir).unwrap();

    let eixt_code = match CommandLineInterface::parse().command {
        Commands::Pack { version_label } => do_pack(version_label, &context),
        Commands::Check => do_check(&context),
        Commands::Combine => {
            // 执行合并前最好先测试一遍
            do_test(&context);
            do_combine(&context)
        },
        Commands::Test => do_test(&context),
    };

    std::process::exit(eixt_code)
}
