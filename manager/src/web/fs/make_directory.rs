use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct MkdirCommand {
    /// 要列目录的路径
    path: String,
}

pub async fn api_make_directory(State(state): State<WebState>, Json(payload): Json<MkdirCommand>) -> Response {
    let path = payload.path;

    // 路径不能为空
    if path.is_empty() {
        return Response::builder()
            .status(500)
            .body(Body::new("parameter 'path' is empty, and it is not allowed.".to_string()))
            .unwrap();
    }

    let file = state.app_context.workspace_dir.join(path);

    println!("make_directory: {:?}", file);

    if file.exists() || file.is_dir() {
        return Response::builder()
            .status(500)
            .body(Body::new("directory has already existed.".to_string()))
            .unwrap();
    }

    tokio::fs::create_dir(&file).await.unwrap();

    Response::builder()
        .status(200)
        .body(Body::empty())
        .unwrap()
}