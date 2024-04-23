//! 打包新版本
//! 
//! 打包过程：
//! 
//! 1. 读取所有历史版本，并推演出上个版本的文件状态，用于和工作空间目录对比生成文件差异
//! 2. 将所有“覆盖的文件”的数据和元数据写入到更新包中，同时更新元数据中每个文件的偏移值
//! 3. 更新索引文件

use std::rc::Weak;

use mcpatch_shared::data::index_file::IndexFile;
use mcpatch_shared::data::index_file::VersionIndex;
use mcpatch_shared::data::version_meta::VersionMeta;
use mcpatch_shared::data::version_meta_group::VersionMetaGroup;

use crate::common::tar_reader::TarReader;
use crate::common::tar_writer::TarWriter;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::AppContext;

/// 执行新版本打包
pub fn do_pack(version_label: String, ctx: &AppContext) -> i32 {
    let mut index_file = IndexFile::load_from_file(&ctx.index_file);

    if index_file.contains(&version_label) {
        println!("版本号已经存在: {}", version_label);
        return 2;
    }

    let mut history = HistoryFile::new_dir("workspace_root", Weak::new());

    // 读取现有更新包，并复现在history上
    for v in &index_file {
        let mut reader = TarReader::new(ctx.public_dir.join(&v.filename));
        let meta_group = reader.read_metadata_group(v.offset, v.len);

        for meta in &meta_group {
            history.replay_operations(&meta);
        }
    }

    // 对比文件
    let disk_file = DiskFile::new(ctx.workspace_dir.clone(), Weak::new());
    let diff = Diff::diff(&disk_file, &history, Some(&ctx.config.filter_rules));

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

        writer.add_file(open, f.len(), &path, &version_label);
    }

    // 写入元数据
    let meta = VersionMeta::new(version_label.clone(), "没有写更新记录".to_owned(), diff.to_file_changes());
    let meta_group = VersionMetaGroup::with_one(meta);
    let meta_info = writer.finish(meta_group);

    // 更新索引文件
    index_file.add(VersionIndex {
        label: version_label.to_owned(),
        filename: version_filename,
        offset: meta_info.offset,
        len: meta_info.length,
        hash: "no hash".to_owned(),
    });
    index_file.save(&ctx.index_file);

    0
}
