use axum::body::Body;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

pub async fn api_upload(State(state): State<WebState>, headers: HeaderMap, body: Body) -> Response {
    let path = match headers.get("path") {
        Some(ok) => ok.to_str().unwrap(),
        None => return PublicResponseBody::<()>::err("no filed 'path' is found in headers."),
    };

    // 对path进行url解码
    let path = url::form_urlencoded::parse(path.as_bytes())
        .map(|(_key, value)| value)
        .collect::<Vec<_>>()
        .join("");

    // 路径不能为空
    if path.is_empty() {
        return PublicResponseBody::<()>::err("parameter 'path' is empty, and it is not allowed.");
    }

    let file = state.config.workspace_dir.join(path);

    println!("upload: {:?}", file);

    if file.is_dir() {
        return PublicResponseBody::<()>::err("file is not writable.");
    }

    // 自动创建上级目录
    tokio::fs::create_dir_all(file.parent().unwrap()).await.unwrap();
    
    let mut f = tokio::fs::File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file)
        .await
        .unwrap();

    let mut body_stream = body.into_data_stream();

    while let Some(frame) = body_stream.next().await {
        let frame = match frame {
            Ok(ok) => ok,
            Err(err) => return PublicResponseBody::<()>::err(&format!("err: {:?}", err)),
        };

        f.write_all(&frame).await.unwrap();
    }

    PublicResponseBody::<()>::ok_no_data()
}