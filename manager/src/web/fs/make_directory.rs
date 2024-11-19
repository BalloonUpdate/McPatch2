use std::collections::HashMap;

use axum::body::Body;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Response;

use crate::web::webstate::WebState;

pub async fn api_make_directory(Query(params): Query<HashMap<String, String>>, State(state): State<WebState>) -> Response {
    let path = match params.get("path") {
        Some(ok) => ok.trim().to_owned(),
        None => return Response::builder()
            .status(500)
            .body(Body::new("parameter 'path' is missing.".to_string()))
            .unwrap(),
    };

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