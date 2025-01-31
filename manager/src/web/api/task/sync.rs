use std::time::UNIX_EPOCH;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use crate::upload::file_list_cache::FileListCache;
use crate::upload::s3::S3Target;
use crate::upload::webdav::WebdavTarget;
use crate::upload::UploadTarget;
use crate::web::webstate::WebState;

/// 同步public目录
pub async fn api_uploa(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let wait = headers.get("wait").is_some();

    state.clone().te.lock().await
        .try_schedule(wait, state.clone(), move || do_upload(state)).await
}

fn do_upload(state: WebState) -> u8 {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    runtime.block_on(async_upload(state));
    
    0
}

async fn async_upload(state: WebState) -> u8 {
    let webdav_config = state.config.webdav.clone();
    let s3_config = state.config.s3.clone();

    // 先上传webdav
    if webdav_config.enabled {
        if let Err(err) = upload("webdav", state.clone(), FileListCache::new(WebdavTarget::new(webdav_config).await)).await {
            state.console.log_error(err);
            return 1;
        }
    }

    // 再上传s3
    if s3_config.enabled {
        if let Err(err) = upload("s3", state.clone(), FileListCache::new(S3Target::new(s3_config).await)).await {
            state.console.log_error(err);
            return 1;
        }
    }

    0
}

async fn upload(name: &str, state: WebState, mut target: impl UploadTarget) -> Result<(), String> {
    let console = &state.console;

    console.log_debug("收集本地文件列表...");
    let local = get_local(&state).await;

    console.log_debug(format!("收集 {} 上的文件列表...", name));
    let remote = target.list().await?;

    console.log_debug("计算文件列表差异...");

    // 寻找上传/覆盖的文件
    let mut need_upload = Vec::new();
    
    for (f, mtime) in &local {
        if remote.iter().any(|e| &e.0 == f && e.1.abs_diff(*mtime) < 3) {
            continue;
        }

        need_upload.push(f.clone());
    }

    // 寻找删除的文件
    let mut need_delete = Vec::new();

    for (f, _) in &remote {
        if local.iter().all(|e| &e.0 != f) {
            need_delete.push(f.clone());
        }
    }

    // 上传文件
    for f in &need_upload {
        console.log_debug(format!("上传文件: {}", f));

        target.upload(&f, state.app_path.public_dir.join(&f)).await?;
    }

    // 删除文件
    for f in &need_delete {
        console.log_debug(format!("删除文件: {}", f));
        
        target.delete(&f).await?;
    }

    console.log_info("文件同步完成");

    Ok(())
}

async fn get_local(state: &WebState) -> Vec<(String, u64)> {
    let mut dir = tokio::fs::read_dir(&state.app_path.public_dir).await.unwrap();

    let mut files = Vec::new();

    while let Some(entry) = dir.next_entry().await.unwrap() {
        let file = entry.file_name().to_str().unwrap().to_owned();
        let mtime = entry.metadata().await.unwrap().modified().unwrap();

        files.push((file, mtime.duration_since(UNIX_EPOCH).unwrap().as_secs()));
    }

    files
}