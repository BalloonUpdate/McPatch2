use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::response::Response;
use tokio::sync::Mutex;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

pub struct TaskExecutor {
    busy_flag: Arc<Mutex<bool>>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        Self {
            busy_flag: Arc::new(Mutex::new(false))
        }
    }

    pub async fn try_schedule<F>(&self, wait: bool, state: WebState, f: F) -> Response where 
        F: FnOnce() -> u8, 
        F: Send + 'static 
    {
        if self.is_busy().await {
            return PublicResponseBody::<()>::err("it is busy now")
        }

        // 先标记已读
        state.console.lock().await.get_logs(true);

        let code = self.schedule(wait, f).await;

        if !wait {
            return PublicResponseBody::<()>::ok_no_data();
        }

        let code = code.unwrap();
        
        let mut buf = String::with_capacity(1024);
    
        for log in state.console.lock().await.get_logs(false) {
            buf += &log.content;
            buf += "\n";
        }

        Response::builder()
            .status(if code == 0 { 200 } else { 500 })
            .body(Body::new(buf))
            .unwrap()
    }

    async fn is_busy(&self) -> bool {
        *self.busy_flag.lock().await
    }

    async fn schedule<F>(&self, wait: bool, f: F) -> Option<u8> where
        F: FnOnce() -> u8,
        F: Send + 'static
    {
        assert!(!self.is_busy().await);

        *self.busy_flag.lock().await = true;

        let busy_clone = self.busy_flag.clone();
        let returns = Arc::new(Mutex::new(0u8));

        let returns2 = returns.clone();
        let handle = std::thread::Builder::new()
            .name("mcpatch-task".into())
            .spawn(move || {
                let value = f();
                *returns2.blocking_lock() = value;
                *busy_clone.blocking_lock() = false;
            })
            .unwrap();

        if !wait {
            return None;
        }

        while !handle.is_finished() {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        let v = *returns.lock().await.deref();

        Some(v)
    }
}