//! mcpatch2管理端第二版

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use clap::Subcommand;
use shared::utility::is_running_under_cargo;
use serde::Deserialize;

use crate::subcommand::check::do_check;
use crate::subcommand::combine::do_combine;
use crate::subcommand::pack::do_pack;
use crate::subcommand::revert::do_revert;
use crate::subcommand::serve::do_serve;
use crate::subcommand::test::do_test;

pub mod utility;
pub mod subcommand;
pub mod diff;
pub mod common;
pub mod upload;

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

    /// 还原工作空间目录的修改
    Revert,

    /// 运行内置服务端
    Serve {
        /// 端口
        #[arg(default_value_t = 0)]
        port: u16,
    },
}

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

    // 如果不带启动参数，就进入交互式模式
    // 如果带了启动参数，就进入命令行模式
    let exit_code = match std::env::args().len() > 1 {
        true => commandline_workmode(),
        false => interactive_workmode(),
    };

    std::process::exit(exit_code)
}

/// 命令行模式，每次只运行一个命令
fn commandline_workmode() -> i32 {
    handle_command(CommandLineInterface::parse())
}

/// 交互式模式，可以重复运行命令
fn interactive_workmode() -> i32 {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut buf = String::with_capacity(1024);

    loop {
        print!("> ");
        stdout.flush().unwrap();
        
        buf.clear();
        buf += &format!("\"{}\" ", std::env::args().next().unwrap());
        let _len = match stdin.read_line(&mut buf) {
            Ok(len) => len,
            Err(_) => break,
        };

        let args = buf.trim().split(" ").map(|e| OsString::from_str(e).unwrap()).collect::<Vec<_>>();

        match CommandLineInterface::try_parse_from(args) {
            Ok(cmd) => { handle_command(cmd); },
            Err(err) => { println!("\n\n {}", err); },
        };
    }

    0
}

fn handle_command(cmd: CommandLineInterface) -> i32 {
    let context = AppContext::new();

    std::fs::create_dir_all(&context.workspace_dir).unwrap();
    
    match cmd.command {
        Commands::Pack { version_label } => do_pack(version_label, &context),
        Commands::Check => do_check(&context),
        Commands::Combine => do_combine(&context),
        Commands::Test => do_test(&context),
        Commands::Revert => do_revert(&context),
        Commands::Serve { port, .. } => do_serve(port, &context),
    }
}