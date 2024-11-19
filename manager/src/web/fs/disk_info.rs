use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use serde::Serialize;

use crate::web::webstate::WebState;

#[derive(Serialize)]
pub struct DiskInfo {
    pub total: u64,
    pub used: u64,
}

pub async fn api_disk_info(State(_state): State<WebState>) -> Response {
    let content = serde_json::to_string_pretty(&DiskInfo {
        total: 100000,
        used: 30000,
    }).unwrap();

    Response::builder()
        .status(200)
        .body(Body::new(content))
        .unwrap()
}