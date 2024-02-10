use std::rc::Weak;

use crate::common::version_reader::VersionReader;
use crate::common::version_writer::VersionWriter;
use crate::AppContext;
use crate::data::index_file::IndexFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::diff::rule_filter::RuleFilter;

pub fn do_pack(version_label: String, ctx: AppContext) -> i32 {
    let mut index_file = IndexFile::load(&ctx.index_file_internal);
    if index_file.contains_label(&version_label) {
        println!("版本号已经存在: {}", version_label);
        return 2;
    }

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

    if !diff.has_diff() {
        println!("目前工作目录还没有任何文件修改");
        return 1;
    }

    println!("{:#?}", diff);

    // 创建新的更新包，将所有文件修改写进去
    std::fs::create_dir_all(&ctx.public_dir).unwrap();
    let version_file = ctx.public_dir.join(&format!("{}.tar", version_label));
    let mut writer = VersionWriter::new(&version_file);
    writer.write_diff(&diff, &ctx.working_dir);

    // 更新索引文件
    index_file.versions.push(version_label.to_owned());
    index_file.save(&ctx.index_file_internal);

    0
}
