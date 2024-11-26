use std::rc::Weak;

use axum::extract::State;
use axum::response::Response;
use shared::data::index_file::IndexFile;

use crate::common::tar_reader::TarReader;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::web::webstate::WebState;

/// 检查工作空间目录的文件修改情况，类似于git status命令
pub async fn api_check(State(state): State<WebState>) -> Response {
    state.clone().te.lock().await
        .try_schedule(move || do_check(state)).await
}

fn do_check(state: WebState) {
    let config = state.config;
    let mut console = state.console.blocking_lock();

    // 读取现有更新包，并复现在history上
    let index_file = IndexFile::load_from_file(&config.index_file);

    console.log_debug("正在读取数据");

    let mut history = HistoryFile::new_empty();

    for v in &index_file {
        let mut reader = TarReader::new(config.public_dir.join(&v.filename));
        let meta_group = reader.read_metadata_group(v.offset, v.len);

        for meta in meta_group {
            history.replay_operations(&meta);
        }
    }

    // 对比文件
    console.log_debug("正在扫描文件更改");

    let exclude_rules = &config.config.blocking_lock().core.exclude_rules;
    let disk_file = DiskFile::new(config.workspace_dir.clone(), Weak::new());
    let diff = Diff::diff(&disk_file, &history, Some(&exclude_rules));

    // 输出文件差异
    console.log_info(format!("{:#?}", diff));
    console.log_info(format!("{}", diff));
}
