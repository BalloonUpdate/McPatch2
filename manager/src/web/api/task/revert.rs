use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use crate::task::revert::task_revert;
use crate::web::webstate::WebState;

/// 恢复工作空间目录到未修改的时候
/// 
/// 有时可能修改了工作空间目录下的文件，但是觉得不满意，想要退回未修改之前，那么可以使用revert命令
pub async fn api_revert(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let wait = headers.get("wait").is_some();

    state.clone().te.lock().await
        .try_schedule(wait, state.clone(), move || do_revert(state)).await
}

pub fn do_revert(state: WebState) -> u8 {
    task_revert(&state.apppath, &state.config, &state.console)
}