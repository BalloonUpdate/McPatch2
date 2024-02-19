use std::rc::Weak;

use crate::common::tar_reader::TarReader;
use crate::data::index_file::IndexFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::AppContext;

/// 检查工作空间目录的文件修改情况，类似于git status命令
pub fn do_check(ctx: &AppContext) -> i32 {
    let index_file = IndexFile::load(&ctx.index_file_internal);
    
    let mut history = HistoryFile::new_empty();

    // 读取现有更新包，并复现在history上
    for v in &index_file {
        let mut reader = TarReader::new(ctx.public_dir.join(&v.filename));
        let meta_group = reader.read_metadata_group(v.offset, v.len);

        for meta in meta_group {
            history.replay_operations(&meta);
        }
    }

    // 对比文件
    let disk_file = DiskFile::new(ctx.workspace_dir.clone(), Weak::new());
    let diff = Diff::diff(&disk_file, &history, Some(&ctx.config.filter_rules));

    // 输出文件差异
    println!("{:#?}", diff);
    println!("{}", diff);

    0
}