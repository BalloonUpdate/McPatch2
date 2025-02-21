use axum::body::Body;
use axum::extract::Path;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use reqwest::StatusCode;
use tokio::io::AsyncSeekExt;

use crate::utility::filename_ext::GetFileNamePart;
use crate::utility::partial_read::PartialAsyncRead;
use crate::web::webstate::WebState;

pub async fn api_public(State(state): State<WebState>, headers: HeaderMap, Path(path): Path<String>) -> Response {
    println!("+public: {}", path);

    let path = state.apppath.public_dir.join(path);

    if !path.is_file() {
        return Response::builder().status(404).body(Body::empty()).unwrap();
    }

    let range = headers.get("range")
        // 拿出range的值
        .map(|e| e.to_str().unwrap())
        // 检查是否以bytes开头
        .filter(|e| e.starts_with("bytes="))
        // 提取出bytes=后面的部分
        .map(|e| e["bytes=".len()..].split("-"))
        // 提取出开始字节和结束字节
        .and_then(|mut e| Some((e.next()?, e.next()?)))
        // 解析开始字节和结束字节
        .and_then(|e| Some((u64::from_str_radix(e.0, 10).ok()?, u64::from_str_radix(e.1, 10).ok()? + 1)))
        // 开始和结束不能都等于0
        .filter(|e| e != &(0, 0))
        // 转换成range
        .map(|e| e.0..e.1);

    // 检查range参数
    if let Some(range) = &range {
        if range.end < range.start {
            return Response::builder().status(403).body(Body::from("incorrect range")).unwrap();
        }
    }

    let metadata = tokio::fs::metadata(&path).await.unwrap();

    let mut file = tokio::fs::File::options()
        .read(true)
        .open(&path)
        .await
        .unwrap();

    if let Some(range) = &range {
        file.seek(std::io::SeekFrom::Start(range.start)).await.unwrap();
    }

    let len = match &range {
        Some(range) => range.end - range.start,
        None => metadata.len(),
    };

    let file = tokio_util::io::ReaderStream::new(PartialAsyncRead::new(file, len));

    let mut builder = Response::builder();

    builder = builder.header(axum::http::header::CONTENT_TYPE, "application/octet-stream");
    builder = builder.header(axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", path.filename()));
    builder = builder.header(axum::http::header::CONTENT_LENGTH, format!("{}", len));

    if let Some(range) = &range {
        builder = builder.header(axum::http::header::CONTENT_RANGE, format!("{}-{}/{}", range.start, range.end - 1, metadata.len()));
        builder = builder.status(StatusCode::PARTIAL_CONTENT);
    }

    builder.body(Body::from_stream(file)).unwrap()
}
