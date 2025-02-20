use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use crate::task::sync::task_upload;
use crate::web::webstate::WebState;

/// 同步public目录
pub async fn api_upload_api(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let wait = headers.get("wait").is_some();

    state.clone().te.lock().await
        .try_schedule(wait, state.clone(), move || do_upload(state)).await
}

fn do_upload(state: WebState) -> u8 {
    task_upload(&state.apppath, &state.config, &state.console)
}
