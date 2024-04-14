pub mod global_config;
pub mod utility;
pub mod error;
pub mod log;
pub mod work;
pub mod network;

use std::path::Path;
use std::path::PathBuf;

use crate::global_config::GlobalConfig;
use crate::log::add_log_handler;
use crate::log::log_info;
use crate::log::set_log_prefix;
use crate::log::ConsoleHandler;
use crate::log::FileHandler;
use crate::log::MessageLevel;
use crate::utility::is_running_under_cargo;
use crate::work::work;

pub struct AppContext {
    pub working_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub public_dir: PathBuf,
    pub index_file: PathBuf,
    pub config: GlobalConfig
}

pub struct StartupParameter {
    pub graphic_mode: bool,
    pub standalone_progress: bool,
    pub disable_log_file: bool,
    // pub external_config_file: String,
}

pub async fn run(params: StartupParameter) {
    let working_dir = get_working_dir().await;
    let executable_dir = get_executable_dir().await;
    let global_config = GlobalConfig::load(&executable_dir.join("mcpatch.yml"));
    let base_dir = get_base_dir(&global_config).await.unwrap();

    let log_file_path = match params.graphic_mode {
        true => executable_dir.join("mcpatch.log"),
        false => executable_dir.join("mcpatch.log.txt"),
    };

    // 初始化文件日志记录器
    if !params.disable_log_file {
        add_log_handler(Box::new(FileHandler::new(&log_file_path)));
    }

    // 初始化stdout日志记录器
    let console_log_level = match is_running_under_cargo() {
        true => match params.graphic_mode || params.disable_log_file {
            true => MessageLevel::Debug,
            false => MessageLevel::Info, // 不需要显示太详细
        },
        false => MessageLevel::Debug,
    };
    add_log_handler(Box::new(ConsoleHandler::new(console_log_level)));

    // 没有独立进程的话需要加上日志前缀，好方便区分
    if !params.standalone_progress {
        set_log_prefix("Mcpatch");
    }

    // 打印运行环境信息
    let gm = if params.graphic_mode { "yes" } else { "no" };
    let sp = if params.standalone_progress { "yes" } else { "no" };
    log_info(&format!("graphic_mode: {gm}, standalone_process: {sp}"));
    log_info(&format!("base directory: {}", base_dir.to_str().unwrap()));
    log_info(&format!("work directory: {}", working_dir.to_str().unwrap()));
    log_info(&format!("prog directory: {}", executable_dir.to_str().unwrap()));

    // todo: localization

    // apply theme

    work(&working_dir, &executable_dir, &base_dir, &global_config, &log_file_path).await.unwrap();
}

/// 获取更新起始目录
async fn get_base_dir(global_config: &GlobalConfig) -> Result<PathBuf, String> {
    let working_dir = get_working_dir().await;

    if is_running_under_cargo() {
        return Ok(working_dir);
    }

    // 智能搜索.minecraft文件夹
    if global_config.base_path.is_empty() {
        let mut current = &working_dir as &Path;

        for _ in 0..7 {
            let detect = tokio::fs::try_exists(current.join(".minecraft")).await;

            match detect {
                Ok(found) => {
                    if found {
                        return Ok(current.to_owned());
                    }

                    current = match current.parent() {
                        Some(parent) => parent,
                        None => break,
                    }
                },
                Err(_) => break,
            }
        }

        return Err(".minecraft not found".into());
    }

    let base_dir = working_dir.join(&global_config.base_path);
    tokio::fs::create_dir_all(&base_dir).await.unwrap();
    Ok(base_dir)
}

/// 获取可执行文件所在目录
async fn get_executable_dir() -> PathBuf {
    if is_running_under_cargo() {
        get_working_dir().await
    } else {
        let exe = std::env::args().next().unwrap();
        PathBuf::from(exe).parent().unwrap().to_owned()
    }
}

/// 获取工作目录
async fn get_working_dir() -> PathBuf {
    let mut working_dir = std::env::current_dir().unwrap();
        
    if is_running_under_cargo() {
        working_dir = working_dir.join("test");
    }

    tokio::fs::create_dir_all(&working_dir).await.unwrap();
    working_dir
}