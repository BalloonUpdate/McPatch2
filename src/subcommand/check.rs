use std::rc::Weak;

use crate::common::tar_reader::TarReader;
use crate::data::index_file::IndexFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::AppContext;

pub fn do_check(ctx: AppContext) -> i32 {
    let index_file = IndexFile::load(&ctx.index_file_internal);
    
    let mut history_file = HistoryFile::new_dir("workspace_root", Weak::new());

    // 读取现有更新包，并复现在history_file上
    for v in &index_file {
        let mut reader = TarReader::new(ctx.public_dir.join(&v.filename));
        for meta in &reader.read_metadata_group(v.offset, v.len) {
            history_file.replay_operations(&meta);
        }
    }

    // 对比文件
    let disk_file = DiskFile::new(ctx.workspace_dir.clone(), None);
    let diff = Diff::diff(&disk_file, &history_file, Some(&ctx.config.filter_rules));

    println!("{:#?}", diff);
    println!("{}", diff);

    0
}