use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct DeleteCommand {
    /// 要列目录的路径
    path: String,
}

pub async fn api_delete(State(state): State<WebState>, Json(payload): Json<DeleteCommand>) -> Response {
    let path = payload.path;

    // 路径不能为空
    if path.is_empty() {
        return Response::builder()
            .status(500)
            .body(Body::new("parameter 'path' is empty, and it is not allowed.".to_string()))
            .unwrap();
    }

    let file = state.config.workspace_dir.join(path);

    if !file.exists() {
        return Response::builder()
            .status(500)
            .body(Body::new("file not exists.".to_string()))
            .unwrap();
    }

    println!("delete: {:?}", file);
    
    if file.is_dir() {
        match tokio::fs::remove_dir_all(&file).await {
            Ok(_) => (),
            Err(err) => return Response::builder()
                .status(500)
                .body(Body::new(format!("{:?}", err)))
                .unwrap(),
        }
    } else {
        match tokio::fs::remove_file(&file).await {
            Ok(_) => (),
            Err(err) => return Response::builder()
                .status(500)
                .body(Body::new(format!("{:?}", err)))
                .unwrap(),
        }
    }

    Response::builder()
        .status(200)
        .body(Body::empty())
        .unwrap()
}