use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 要列目录的路径
    path: String,
}

pub async fn api_make_directory(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    let path = payload.path;

    // 路径不能为空
    if path.is_empty() {
        return PublicResponseBody::<()>::err("parameter 'path' is empty, and it is not allowed.");
    }

    let file = state.app_path.workspace_dir.join(path);

    println!("make_directory: {:?}", file);

    if file.exists() || file.is_dir() {
        return PublicResponseBody::<()>::err("directory has already existed.");
    }

    tokio::fs::create_dir(&file).await.unwrap();

    PublicResponseBody::<()>::ok_no_data()
}