use std::time::SystemTime;

use axum::body::Body;
use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;

use crate::web::file_status::SingleFileStatus;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct ListCommand {
    /// 要列目录的路径
    path: String,
}

#[derive(Serialize)]
pub struct File {
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
    pub ctime: u64,
    pub mtime: u64,
    pub state: String,
}

pub async fn api_list(State(state): State<WebState>, Json(payload): Json<ListCommand>) -> Response {
    let path = payload.path;

    // // 路径不能为空
    // if path.is_empty() {
    //     return Response::builder()
    //         .status(500)
    //         .body(Body::new("parameter 'path' is empty, and it is not allowed.".to_string()))
    //         .unwrap();
    // }

    let mut status = state.status.lock().await;

    let file = state.config.workspace_dir.join(&path);

    println!("list: {:?}", file);

    if !file.exists() || !file.is_dir() {
        return Response::builder()
            .status(500)
            .body(Body::new("file not exists.".to_string()))
            .unwrap();
    }

    let mut response = Vec::<File>::new();

    let mut read_dir = tokio::fs::read_dir(&file).await.unwrap();

    while let Some(entry) = read_dir.next_entry().await.unwrap() {
        let is_directory = entry.file_type().await.unwrap().is_dir();
        let metadata = entry.metadata().await.unwrap();

        let relative_path = entry.path().strip_prefix(&state.config.workspace_dir).unwrap().to_str().unwrap().replace("\\", "/");

        // println!("relative: {:?}", relative_path);

        let st = status.get_file_status(&relative_path);

        response.push(File {
            name: entry.file_name().to_str().unwrap().to_string(),
            is_directory,
            size: if is_directory { 0 } else { metadata.len() },
            ctime: metadata.created().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            mtime: metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            state: match st {
                SingleFileStatus::Keep => "keep".to_owned(),
                SingleFileStatus::Added => "added".to_owned(),
                SingleFileStatus::Modified => "modified".to_owned(),
                SingleFileStatus::Missing => "missing".to_owned(),
                SingleFileStatus::Gone => "gone".to_owned(),
                SingleFileStatus::Come => "come".to_owned(),
            },
        });
    }

    let content = serde_json::to_string(&response).unwrap();

    Response::builder()
        .status(200)
        .body(Body::new(content))
        .unwrap()
}