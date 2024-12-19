use std::time::SystemTime;

use axum::extract::State;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;

use crate::web::api::PublicResponseBody;
use crate::web::file_status::SingleFileStatus;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 要列目录的路径
    path: String,
}

#[derive(Serialize)]
pub struct ResponseData {
    pub files: Vec<File>,
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

#[axum::debug_handler]
pub async fn api_list(State(state): State<WebState>, Json(payload): Json<RequestBody>) -> Response {
    let mut status = state.status.lock().await;

    let dir = state.app_path.working_dir.join(&payload.path);

    // println!("list: {:?}", dir);

    if !dir.exists() || !dir.is_dir() {
        return PublicResponseBody::<ResponseData>::err("directory not exists.");
    }

    let mut files = Vec::<File>::new();

    let mut read_dir = tokio::fs::read_dir(&dir).await.unwrap();

    while let Some(entry) = read_dir.next_entry().await.unwrap() {
        let is_directory = entry.file_type().await.unwrap().is_dir();
        let metadata = entry.metadata().await.unwrap();

        let status = match entry.path().strip_prefix(&state.app_path.workspace_dir) {
            Ok(ok) => status.get_file_status(&ok.to_str().unwrap().replace("\\", "/")).await,
            Err(_) => SingleFileStatus::Keep,
        };

        // let relative_path = entry.path().strip_prefix(&state.app_path.working_dir).unwrap().to_str().unwrap().replace("\\", "/");
        // println!("relative: {:?}", relative_path);

        files.push(File {
            name: entry.file_name().to_str().unwrap().to_string(),
            is_directory,
            size: if is_directory { 0 } else { metadata.len() },
            ctime: metadata.created().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            mtime: metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            state: match status {
                SingleFileStatus::Keep => "keep".to_owned(),
                SingleFileStatus::Added => "added".to_owned(),
                SingleFileStatus::Modified => "modified".to_owned(),
                SingleFileStatus::Missing => "missing".to_owned(),
                SingleFileStatus::Gone => "gone".to_owned(),
                SingleFileStatus::Come => "come".to_owned(),
            },
        });
    }
    
    PublicResponseBody::<ResponseData>::ok(ResponseData { files })
}