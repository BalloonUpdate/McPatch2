use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use crate::task::combine::task_combine;
use crate::web::webstate::WebState;

// 执行更新包合并操作
pub async fn api_combine(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let wait = headers.get("wait").is_some();

    state.clone().te.lock().await
        .try_schedule(wait, state.clone(), move || do_combine(state)).await
}

fn do_combine(state: WebState) -> u8 {
    task_combine(&state.apppath, &state.config, &state.console)
}
