use std::collections::HashMap;

use axum::body::Body;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Response;
use tokio_util::io::ReaderStream;

use crate::web::webstate::WebState;

pub async fn api_download(Query(params): Query<HashMap<String, String>>, State(state): State<WebState>) -> Response {
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

    if !file.exists() || !file.is_file() {
        return Response::builder()
            .status(500)
            .body(Body::new("file not exists.".to_string()))
            .unwrap();
    }

    println!("download: {:?}", file);

    let stream = tokio::fs::File::options()
        .read(true)
        .open(file)
        .await
        .unwrap();

    let stream = ReaderStream::new(stream);

    Response::builder()
        .status(200)
        .body(Body::from_stream(stream))
        .unwrap()
}