//! mcpatch2管理端第二版

use std::ffi::OsString;
use std::io::Write;
use std::str::FromStr;

use clap::Parser;
use clap::Subcommand;

use crate::app_path::AppPath;
use crate::builtin_server::start_builtin_server;
use crate::config::Config;
use crate::task::check::task_check;
use crate::task::combine::task_combine;
use crate::task::pack::task_pack;
use crate::task::revert::task_revert;
use crate::task::test::task_test;
use crate::web::log::Console;
use crate::web::serve_web;

pub mod utility;
pub mod diff;
pub mod core;
pub mod config;
pub mod web;
pub mod builtin_server;
pub mod upload;
pub mod app_path;
pub mod task;

#[derive(Parser)]
struct CommandLineInterface {
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

    /// 运行私有协议服务端
    Serve,

    /// 运行webui
    Webui, 
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async move {
        let apppath = AppPath::new();
        let config = Config::load(&apppath).await;
        let console = Console::new_cli();

        match std::env::args().len() > 1 {
            // 如果带了启动参数，就进入命令行模式
            true => commandline_mode(apppath, config, console).await,
            
            // 如果不带启动参数，就进入交互式模式
            false => interactive_mode(apppath, config, console).await,
        }
    });
}

/// 命令行模式，每次只运行一个命令
async fn commandline_mode(apppath: AppPath, config: Config, console: Console) -> i32 {
    handle_command(&apppath, &config, &console, CommandLineInterface::parse()).await
}

/// 交互式模式，可以重复运行命令
async fn interactive_mode(apppath: AppPath, config: Config, console: Console) -> i32 {
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
            Ok(cmd) => { handle_command(&apppath, &config, &console, cmd).await; },
            Err(err) => { println!("\n\n {}", err); },
        };
    }

    0
}

async fn handle_command(apppath: &AppPath, config: &Config, console: &Console, cmd: CommandLineInterface) -> i32 {
    let result = match cmd.command {
        Commands::Pack { version_label } => task_pack(version_label, "".to_owned(), apppath, config, console),
        Commands::Check => task_check(apppath, config, console),
        Commands::Combine => task_combine(apppath, config, console),
        Commands::Test => task_test(apppath, config, console),
        Commands::Revert => task_revert(apppath, config, console),
        Commands::Serve => {
            start_builtin_server(config.clone(), apppath.clone()).await;

            0
        },
        Commands::Webui => {
            serve_web(apppath.clone(), config.clone()).await;

            0
        },
    };

    result as i32
}
