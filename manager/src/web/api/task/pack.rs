use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::task::pack::task_pack;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 新包的版本号
    label: String,

    /// 新包的更新记录
    change_logs: String,
}

/// 打包新版本
pub async fn api_pack(State(state): State<WebState>, headers: HeaderMap, Json(payload): Json<RequestBody>) -> Response {
    let wait = headers.get("wait").is_some();

    state.clone().te.lock().await
        .try_schedule(wait, state.clone(), move || do_check(payload, state)).await
}

fn do_check(payload: RequestBody, state: WebState) -> u8 {
    let version_label = payload.label;
    let change_logs = payload.change_logs;

    task_pack(version_label, change_logs, &state.apppath, &state.config, &state.console)
}