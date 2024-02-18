use std::rc::Weak;

use crate::common::tar_reader::TarReader;
use crate::common::tar_writer::TarWriter;
use crate::data::index_file::VersionIndex;
use crate::data::version_meta::VersionMeta;
use crate::data::version_meta_group::VersionMetaGroup;
use crate::diff::abstract_file::AbstractFile;
use crate::AppContext;
use crate::data::index_file::IndexFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;

pub fn do_pack(version_label: String, ctx: AppContext) -> i32 {
    let mut index_file = IndexFile::load(&ctx.index_file_internal);
    if index_file.contains_label(&version_label) {
        println!("版本号已经存在: {}", version_label);
        return 2;
    }

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

    if !diff.has_diff() {
        println!("目前工作目录还没有任何文件修改");
        return 1;
    }

    println!("{:#?}", diff);

    // 创建新的更新包，将所有文件修改写进去
    std::fs::create_dir_all(&ctx.public_dir).unwrap();
    let version_filename = format!("{}.tar", version_label);
    let version_file = ctx.public_dir.join(&version_filename);
    let mut writer = TarWriter::new(&version_file);

    // 写入每个更新的文件数据
    for f in &diff.updated_files {
        let path = f.path().to_owned();
        let disk_file = ctx.workspace_dir.join(&path);
        let open = std::fs::File::options().read(true).open(disk_file).unwrap();

        writer.write_file(open, f.len(), &path);
    }

    // 写入元数据
    let meta = VersionMeta::new(version_label.clone(), "no logs".to_owned(), &diff);
    let meta_group = VersionMetaGroup::with_one(meta);
    let meta_info = writer.finish(meta_group);

    // 更新索引文件
    index_file.add_index(VersionIndex {
        label: version_label.to_owned(),
        filename: version_filename,
        offset: meta_info.offset,
        len: meta_info.length,
        hash: "no hash".to_owned(),
    });
    index_file.save(&ctx.index_file_internal);

    0
}
