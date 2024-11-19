use std::collections::HashMap;

use axum::body::Body;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Response;

use crate::web::webstate::WebState;

pub async fn api_delete(Query(params): Query<HashMap<String, String>>, State(state): State<WebState>) -> Response {
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