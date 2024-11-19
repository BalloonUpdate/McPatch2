use std::sync::Arc;

use axum::body::Body;
use axum::response::Response;
use tokio::sync::Mutex;

pub struct TaskExecutor {
    busy_flag: Arc<Mutex<bool>>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        Self {
            busy_flag: Arc::new(Mutex::new(false))
        }
    }

    pub async fn try_schedule<F>(&self, f: F) -> Response where 
        F: FnOnce() -> (), 
        F: Send + 'static 
    {
        if self.is_busy().await {
            return Response::builder()
                .status(500)
                .body(Body::new("it is busy now".to_string()))
                .unwrap()
        }

        self.schedule(f).await;

        Response::builder()
            .status(200)
            .body(Body::empty())
            .unwrap()
    }

    async fn is_busy(&self) -> bool {
        *self.busy_flag.lock().await
    }

    async fn schedule<F>(&self, f: F) where
        F: FnOnce() -> (),
        F: Send + 'static
    {
        assert!(!self.is_busy().await);

        *self.busy_flag.lock().await = true;

        let busy_clone = self.busy_flag.clone();

        std::thread::Builder::new()
            .name("mcpatch-task".into())
            .spawn(move || {
                f();
                *busy_clone.blocking_lock() = false;
            })
            .unwrap();
    }
}