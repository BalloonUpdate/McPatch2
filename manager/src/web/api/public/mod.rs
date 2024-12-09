use axum::body::Body;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Response;
use shared::utility::filename_ext::GetFileNamePart;

use crate::web::webstate::WebState;

pub async fn api_public(State(state): State<WebState>, Path(path): Path<String>) -> Response {
    println!("+public: {}", path);

    let path = state.app_path.public_dir.join(path);

    if !path.is_file() {
        return Response::builder().status(404).body(Body::empty()).unwrap();
    }

    let metadata = tokio::fs::metadata(&path).await.unwrap();

    let file = tokio::fs::File::options()
        .read(true)
        .open(&path)
        .await
        .unwrap();

    let file = tokio_util::io::ReaderStream::new(file);

    Response::builder()
        .header(axum::http::header::CONTENT_TYPE, "application/octet-stream")
        .header(axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", path.filename()))
        .header(axum::http::header::CONTENT_LENGTH, format!("{}", metadata.len()))
        .body(Body::from_stream(file))
        .unwrap()
}
