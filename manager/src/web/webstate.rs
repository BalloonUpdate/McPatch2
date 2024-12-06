use std::sync::Arc;

use tokio::sync::Mutex;

use crate::app_path::AppPath;
use crate::config::auth_config::AuthConfig;
use crate::config::Config;
use crate::web::file_status::FileStatus;
use crate::web::log::ConsoleBuffer;
use crate::web::task_executor::TaskExecutor;

#[derive(Clone)]
pub struct WebState {
    pub app_path: AppPath,
    pub config: Config,
    pub auth: AuthConfig,
    pub console: Arc<Mutex<ConsoleBuffer>>,
    pub te: Arc<Mutex<TaskExecutor>>,
    pub status: Arc<Mutex<FileStatus>>,
}

impl WebState {
    pub fn new(app_path: AppPath, config: Config, auth: AuthConfig) -> Self {
        Self {
            app_path: app_path.clone(),
            config: config.clone(),
            auth,
            console: Arc::new(Mutex::new(ConsoleBuffer::new())),
            te: Arc::new(Mutex::new(TaskExecutor::new())),
            status: Arc::new(Mutex::new(FileStatus::new(app_path, config))),
        }
    }
}
