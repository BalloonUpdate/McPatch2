use std::rc::Weak;

use crate::common::version_reader::VersionReader;
use crate::data::index_file::IndexFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::diff::rule_filter::RuleFilter;
use crate::AppContext;

pub fn do_check(ctx: AppContext) -> i32 {
    let index_file = IndexFile::load(&ctx.index_file_internal);
    
    let mut history_file = HistoryFile::new_dir("workspace_root", Weak::new());

    // 读取现有更新包，并复现在history_file上
    for filename in &index_file.versions {
        let mut reader = VersionReader::new(ctx.public_dir.join(filename));
        let meta = reader.read_metadata();
        history_file.replay_operations(&meta);
    }

    // 对比文件
    let disk_file = DiskFile::new(ctx.workspace_dir.clone(), None);
    let rule_filter = RuleFilter::new([""; 0].iter());
    let diff = Diff::diff(&disk_file, &history_file, &rule_filter);


    println!("{:#?}", diff);
    println!("{}", diff);

    0
}