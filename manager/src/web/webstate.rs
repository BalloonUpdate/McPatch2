use std::sync::Arc;

use tokio::sync::Mutex;

use crate::app_path::AppPath;
use crate::config::auth_config::AuthConfig;
use crate::config::Config;
use crate::web::file_status::FileStatus;
use crate::web::log::Console;
use crate::web::task_executor::LongTimeExecutor;

/// 整个web服务共享的上下文对象
#[derive(Clone)]
pub struct WebState {
    pub apppath: AppPath,
    pub config: Config,
    pub auth: AuthConfig,
    pub console: Console,
    pub te: Arc<Mutex<LongTimeExecutor>>,
    pub status: Arc<Mutex<FileStatus>>,
}

impl WebState {
    pub fn new(app_path: AppPath, config: Config, auth: AuthConfig) -> Self {
        Self {
            apppath: app_path.clone(),
            config: config.clone(),
            auth,
            console: Console::new_webui(),
            te: Arc::new(Mutex::new(LongTimeExecutor::new())),
            status: Arc::new(Mutex::new(FileStatus::new(app_path, config))),
        }
    }
}
