use axum::extract::State;
use axum::response::Response;
use serde::Serialize;

use crate::web::api::PublicResponseBody;
use crate::web::log::LogEntry;
use crate::web::webstate::WebState;

#[derive(Serialize)]
pub struct ResponseData {
    /// 返回的日志文本
    pub content: Vec<LogEntry>,
}

pub async fn api_full(State(state): State<WebState>) -> Response {
    let console = &state.console;

    let buf = console.get_logs(true);

    PublicResponseBody::<ResponseData>::ok(ResponseData { content: buf })
}