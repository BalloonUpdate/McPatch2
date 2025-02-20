use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 要删除的文件路径
    path: String,
}

pub async fn api_delete(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    let path = payload.path;

    // 路径不能为空
    if path.is_empty() {
        return PublicResponseBody::<()>::err("parameter 'path' is empty");
    }

    let file = state.apppath.working_dir.join(path);

    if !file.exists() {
        return PublicResponseBody::<()>::err("file not exists.");
    }
    
    if file.is_dir() {
        match tokio::fs::remove_dir_all(&file).await {
            Ok(_) => (),
            Err(err) => return PublicResponseBody::<()>::err(&format!("{:?}", err)),
        }
    } else {
        match tokio::fs::remove_file(&file).await {
            Ok(_) => (),
            Err(err) => return PublicResponseBody::<()>::err(&format!("{:?}", err)),
        }
    }

    // 清除文件状态缓存
    let mut status = state.status.lock().await;
    status.invalidate();

    PublicResponseBody::<()>::ok_no_data()
}