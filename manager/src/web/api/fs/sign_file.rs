use std::time::Duration;
use std::time::SystemTime;

use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 要下载的文件路径
    path: String,
}

#[derive(Serialize)]
pub struct ResponseData {
    /// 文件的签名数据
    signature: String,
}

pub async fn api_sign_file(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    // 路径不能为空
    if payload.path.is_empty() {
        return PublicResponseBody::<ResponseData>::err("parameter 'path' is empty, and it is not allowed.");
    }

    let path = state.apppath.working_dir.join(payload.path);

    if !path.exists() || !path.is_file() {
        return PublicResponseBody::<ResponseData>::err("file not exists.");
    }

    let username = state.auth.username().await;
    let password = state.auth.password().await;

    let relative_path = path.strip_prefix(&state.apppath.working_dir).unwrap().to_str().unwrap().to_owned();
    let expire = SystemTime::now() + Duration::from_secs(2 * 60 * 60);
    let unix_ts = expire.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

    let core_data = format!("{}:{}", relative_path, unix_ts);
    let full_data = format!("{}:{}@{}", core_data, username, password);
    let digest = hash(&full_data);
    let signature = format!("{}:{}", core_data, digest);

    // println!("full_data: {} | signature: {}", full_data, signature);

    PublicResponseBody::<ResponseData>::ok(ResponseData { signature })
}

fn hash(text: &impl AsRef<str>) -> String {
    let hash = Sha256::digest(text.as_ref());
    
    base16ct::lower::encode_string(&hash)
}