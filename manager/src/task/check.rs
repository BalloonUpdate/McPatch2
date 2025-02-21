use std::rc::Weak;

use crate::app_path::AppPath;
use crate::config::Config;
use crate::core::data::index_file::IndexFile;
use crate::core::tar_reader::TarReader;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::web::log::Console;

pub fn task_check(apppath: &AppPath, config: &Config, console: &Console) -> u8 {
    // 读取现有更新包，并复现在history上
    let index_file = IndexFile::load_from_file(&apppath.index_file);

    console.log_debug("正在读取数据");

    let mut history = HistoryFile::new_empty();

    for v in &index_file {
        let mut reader = TarReader::new(apppath.public_dir.join(&v.filename));
        let meta_group = reader.read_metadata_group(v.offset, v.len);

        for meta in meta_group {
            history.replay_operations(&meta);
        }
    }

    // 对比文件
    console.log_debug("正在扫描文件更改");

    let exclude_rules = &config.core.exclude_rules;
    let disk_file = DiskFile::new(apppath.workspace_dir.clone(), Weak::new());
    let diff = Diff::diff(&disk_file, &history, Some(&exclude_rules));

    // 输出文件差异
    console.log_info(format!("{:#?}", diff));
    console.log_info(format!("{}", diff));

    0
}