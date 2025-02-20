use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use crate::task::test::task_test;
use crate::web::webstate::WebState;

/// 执行更新包解压测试
pub async fn api_test(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let wait = headers.get("wait").is_some();

    state.clone().te.lock().await
        .try_schedule(wait, state.clone(), move || do_test(state)).await
}

fn do_test(state: WebState) -> u8 {
    task_test(&state.apppath, &state.config, &state.console)
}