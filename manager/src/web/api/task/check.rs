use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use crate::task::check::task_check;
use crate::web::webstate::WebState;

/// 检查工作空间目录的文件修改情况，类似于git status命令
pub async fn api_status(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let wait = headers.get("wait").is_some();

    state.clone().te.lock().await
        .try_schedule(wait, state.clone(), move || do_status(state)).await
}

fn do_status(state: WebState) -> u8 {
    task_check(&state.apppath, &state.config, &state.console)
}
