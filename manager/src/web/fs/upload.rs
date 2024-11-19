use std::collections::HashMap;

use axum::body::Body;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Response;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

use crate::web::webstate::WebState;

pub async fn api_upload(Query(params): Query<HashMap<String, String>>, State(state): State<WebState>, body: Body) -> Response {
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

    println!("list: {:?}", file);

    if file.is_dir() {
        return Response::builder()
            .status(500)
            .body(Body::new("file is not writable.".to_string()))
            .unwrap();
    }
    
    let mut f = tokio::fs::File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file)
        .await
        .unwrap();

    let mut body = body.into_data_stream();

    while let Some(chunk) = body.next().await {
        match chunk {
            Ok(ok) => {
                f.write_all(&ok).await.unwrap();
            },
            Err(err) => {
                println!("failed to receive data: {:?}", err);
            },
        }
    }
    
    Response::builder()
        .status(200)
        .body(Body::empty())
        .unwrap()
}