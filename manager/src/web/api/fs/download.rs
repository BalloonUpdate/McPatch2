use axum::extract::State;
use axum::response::Response;
use axum::Json;
use base64ct::Encoding;
use serde::Deserialize;
use serde::Serialize;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 要列目录的路径
    path: String,
}

#[derive(Serialize)]
pub struct ResponseData {
    /// 经过base64编码的完整文件内容
    pub content: String,
}

pub async fn api_download(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    // 路径不能为空
    if payload.path.is_empty() {
        return PublicResponseBody::<ResponseData>::err("parameter 'path' is empty, and it is not allowed.");
    }

    let file = state.app_path.working_dir.join(payload.path);

    if !file.exists() || !file.is_file() {
        return PublicResponseBody::<ResponseData>::err("file not exists.");
    }

    // println!("download: {:?}", file);

    let data = tokio::fs::read(&file).await.unwrap();

    let b64 = base64ct::Base64::encode_string(&data);

    PublicResponseBody::<ResponseData>::ok(ResponseData { content: b64 })
}