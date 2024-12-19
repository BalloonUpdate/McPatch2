use axum::body::Body;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Response;

use crate::web::webstate::WebState;

#[cfg(feature = "bundle-webpage")]
static WEBPAGE_DIR: include_dir::Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/../test/webpage");

pub async fn api_webpage(State(state): State<WebState>, Path(path): Path<String>) -> Response {
    respond_file(&path, &state).await
}

pub async fn api_webpage_index(State(state): State<WebState>) -> Response {
    respond_file("", &state).await
}

async fn respond_file(mut path: &str, state: &WebState) -> Response {
    let raw_path = path.to_owned();

    if path == "" {
        path = &state.config.web.index_filename;
    }

    // 当外部文件夹存在时，优先从外部文件夹响应
    if state.app_path.web_dir.exists() {
        println!("+webpage-o /{}", raw_path);

        return respond_from_outer(path, state).await;
    }

    println!("+webpage-i /{}", raw_path);

    // 从文件内部响应
    #[cfg(feature = "bundle-webpage")]
    return respond_from_inner(path, state).await;

    #[allow(unreachable_code)]
    { panic!("the webpdage folder does not exist: {}", state.app_path.web_dir.to_str().unwrap()); }
}

/// 从可执行文件内部响应页面文件请求
#[cfg(feature = "bundle-webpage")]
async fn respond_from_inner(mut path: &str, state: &WebState) -> Response {
    // println!("inner");

    // 文件找不到就尝试访问404文件
    if !WEBPAGE_DIR.contains(path) && !state.config.web.redirect_404.is_empty() {
        path = &state.config.web.redirect_404;
    }

    // 如果还是找不到就返回404了
    if !WEBPAGE_DIR.contains(path) {
        return Response::builder().status(404).body(Body::empty()).unwrap();
    }

    // 下面正常处理请求
    let file = WEBPAGE_DIR.get_file(path).unwrap();

    let contents = file.contents();

    let mime_info = mime_guess::from_path(path).first_or(mime_guess::mime::APPLICATION_OCTET_STREAM);

    return Response::builder()
        .header(axum::http::header::CONTENT_TYPE, mime_info.essence_str())
        .header(axum::http::header::CONTENT_LENGTH, format!("{}", contents.len()))
        .body(Body::from_stream(tokio_util::io::ReaderStream::new(contents)))
        .unwrap();
}

/// 从外部的webpage目录响应页面文件请求
async fn respond_from_outer(path: &str, state: &WebState) -> Response {
    // println!("outer");

    let mut path = state.app_path.web_dir.join(path);

    // 文件找不到就尝试访问404文件
    if !path.is_file() && !state.config.web.redirect_404.is_empty() {
        path = state.app_path.web_dir.join(&state.config.web.redirect_404);
    }

    // 如果还是找不到就返回404了
    if !path.is_file() {
        return Response::builder().status(404).body(Body::empty()).unwrap();
    }
    
    let metadata = tokio::fs::metadata(&path).await.unwrap();

    let file = tokio::fs::File::options()
        .read(true)
        .open(&path)
        .await
        .unwrap();

    let mime_info = mime_guess::from_path(path).first_or(mime_guess::mime::APPLICATION_OCTET_STREAM);

    Response::builder()
        .header(axum::http::header::CONTENT_TYPE, mime_info.essence_str())
        .header(axum::http::header::CONTENT_LENGTH, format!("{}", metadata.len()))
        .body(Body::from_stream(tokio_util::io::ReaderStream::new(file)))
        .unwrap()
}

