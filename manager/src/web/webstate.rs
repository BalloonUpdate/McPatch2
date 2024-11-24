use std::sync::Arc;

use tokio::sync::Mutex;

use crate::config::config::Config;
use crate::web::file_status::FileStatus;
use crate::web::log::ConsoleBuffer;
use crate::web::task_executor::TaskExecutor;
use crate::web::token::Token;

#[derive(Clone)]
pub struct WebState {
    pub config: Config,
    pub console: Arc<Mutex<ConsoleBuffer>>,
    pub te: Arc<Mutex<TaskExecutor>>,
    pub status: Arc<Mutex<FileStatus>>,
    pub token: Arc<Mutex<Token>>,
}

impl WebState {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
            console: Arc::new(Mutex::new(ConsoleBuffer::new())),
            te: Arc::new(Mutex::new(TaskExecutor::new())),
            status: Arc::new(Mutex::new(FileStatus::new(config))),
            token: Arc::new(Mutex::new(Token::default())),
        }
    }
}
