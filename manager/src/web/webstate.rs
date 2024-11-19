use std::sync::Arc;

use tokio::sync::Mutex;

use crate::web::file_status::FileStatus;
use crate::web::log::ConsoleBuffer;
use crate::web::task_executor::TaskExecutor;
use crate::AppContext;

#[derive(Clone)]
pub struct WebState {
    pub app_context: AppContext,
    pub console: Arc<Mutex<ConsoleBuffer>>,
    pub te: Arc<Mutex<TaskExecutor>>,
    pub status: Arc<Mutex<FileStatus>>,
}

impl WebState {
    pub fn new(app_content: AppContext) -> Self {
        Self {
            app_context: app_content.clone(),
            console: Arc::new(Mutex::new(ConsoleBuffer::new())),
            te: Arc::new(Mutex::new(TaskExecutor::new())),
            status: Arc::new(Mutex::new(FileStatus::new(app_content.clone()))),
        }
    }
}