use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::response::Response;
use tokio::sync::Mutex;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

/// 代表长时间任务执行器
pub struct LongTimeExecutor {
    busy: Arc<Mutex<bool>>,
}

impl LongTimeExecutor {
    pub fn new() -> Self {
        Self {
            busy: Arc::new(Mutex::new(false))
        }
    }

    /// 尝试执行一个任务。并直接生成Response对象
    pub async fn try_schedule<F>(&self, wait: bool, state: WebState, f: F) -> Response where 
        F: FnOnce() -> u8, 
        F: Send + 'static 
    {
        // 同时只能有一个任务在运行
        if self.is_busy().await {
            return PublicResponseBody::<()>::err("it is busy now")
        }

        // 先把缓冲区里所有的日志标记为已读
        state.console.get_logs(true);

        // 执行任务
        let code = self.schedule(wait, f).await;

        // 如果不等待的话，就直接返回
        if !wait {
            return PublicResponseBody::<()>::ok_no_data();
        }

        // 拿到任务返回代码
        let code = code.unwrap();
        
        // 收集期间的所有日志输出
        let mut buf = String::with_capacity(1024);
    
        for log in state.console.get_logs(false) {
            buf += &log.content;
            buf += "\n";
        }

        // 将日志输出写到Response里
        Response::builder()
            .status(if code == 0 { 200 } else { 500 })
            .body(Body::new(buf))
            .unwrap()
    }

    /// 当然有任务在执行吗
    async fn is_busy(&self) -> bool {
        *self.busy.lock().await
    }

    /// 执行一个任务
    /// 
    /// + 当`wait`为true时，会等待任务结束后返回，同时携带返回代码
    /// + 当`wait`为false时，会立即返回，没有返回代码
    async fn schedule<F>(&self, wait: bool, f: F) -> Option<u8> where
        F: FnOnce() -> u8,
        F: Send + 'static
    {
        assert!(!self.is_busy().await);

        // 设置busy标记
        *self.busy.lock().await = true;

        let busy_clone = self.busy.clone();
        let returns = Arc::new(Mutex::new(0u8));

        // 准备启动单独线程执行任务
        let returns2 = returns.clone();
        let handle = std::thread::Builder::new()
            .name("mcpatch-task".into())
            .spawn(move || {
                // 执行任务
                let value = f();

                // 保存返回代码
                *returns2.blocking_lock() = value;

                // 设置busy标记
                *busy_clone.blocking_lock() = false;
            })
            .unwrap();

        // 如果不等待，立即返回
        if !wait {
            return None;
        }

        // 等待任务运行结束
        while !handle.is_finished() {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        // 返回返回代码
        let v = *returns.lock().await.deref();

        Some(v)
    }
}