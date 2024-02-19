use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Weak;

use crate::common::file_hash::calculate_hash;
use crate::common::tar_reader::TarReader;
use crate::common::tar_writer::TarWriter;
use crate::data::index_file::IndexFile;
use crate::data::index_file::VersionIndex;
use crate::data::version_meta::FileChange;
use crate::data::version_meta_group::VersionMetaGroup;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::history_file::HistoryFile;
use crate::utility::ext::GetFileNamePart;
use crate::AppContext;

pub const COMBINED_FILENAME: &str = "_combined.tar";

/// 代表新的合并包中的某个文件数据要从哪个旧包中复制过来
struct Location {
    /// 所在的版本
    pub label: String,

    /// 所在的tar包的文件名
    pub filename: String,

    /// 最原始的文件路径（不受后续移动操作的影响）
    pub path: String,

    /// tar包中的文件偏移
    pub offset: u64,

    /// 数据的长度
    pub len: u64,
}

pub fn do_combine(ctx: &AppContext) -> i32 {
    let index_file = IndexFile::load(&ctx.index_file_internal);

    if (&index_file).into_iter().all(|e| e.filename == COMBINED_FILENAME) {
        println!("no data can be combined");
        return 0;
    }

    let mut history_file = HistoryFile::new_dir("workspace_root", Weak::new());
    let mut data_locations = HashMap::<String, Location>::new();

    // 保留所有元数据，最后会合并写入tar包里
    let mut meta_group = VersionMetaGroup::new();

    // 读取现有更新包，并复现在history_file上
    for v in &index_file {
        let mut reader = TarReader::new(ctx.public_dir.join(&v.filename));
        let group = reader.read_metadata_group(v.offset, v.len);
        for meta in &group {
            history_file.replay_operations(&meta);

            // 记录所有文件的数据和来源
            for change in &meta.changes {
                match change {
                    FileChange::UpdateFile { path, offset, len, .. } => {
                        data_locations.insert(path.to_owned(), Location {
                            label: meta.label.clone(),
                            filename: v.filename.to_owned(),
                            path: path.to_owned(),
                            offset: *offset,
                            len: *len,
                        });
                    },
                    FileChange::DeleteFile { path } => {
                        data_locations.remove(path);
                    },
                    FileChange::MoveFile { from, to } => {
                        let hold = data_locations.remove(from).unwrap();
                        data_locations.insert(to.to_owned(), hold);
                    }
                    _ => (),
                }
            }
        }
        meta_group.add_group(group);
    }

    // 生成新的合并包
    let new_tar_file = ctx.public_dir.join("_combined.temp.tar");
    let mut writer = TarWriter::new(&new_tar_file);

    // 写入每个版本里的所有文件数据
    for (_, loc) in &data_locations {
        // 读取原tar包中的文件，然后复制到合并包中
        let mut reader = TarReader::new(ctx.public_dir.join(&loc.filename));
        let read = reader.open_file(loc.offset, loc.len);
        writer.add_file(read, loc.len, &loc.path, &loc.label);
    }

    // 写入元数据
    let meta_loc = writer.finish(meta_group);

    // 更新索引文件
    let new_index_filepath = ctx.public_dir.join("_index.temp.json");
    let mut new_index_file = IndexFile::load(&new_index_filepath);
    for index in &index_file {
        new_index_file.add_index(VersionIndex {
            label: index.label.to_owned(),
            filename: new_tar_file.filename().to_owned(),
            offset: meta_loc.offset,
            len: meta_loc.length,
            hash: "no hash".to_owned(),
        })
    }
    new_index_file.save(&new_index_filepath);

    // 测试合并包
    let mut reader = TarReader::new(&new_tar_file);
    let meta_group = reader.read_metadata_group(meta_loc.offset, meta_loc.length);
    test_combined_package(reader, meta_group);
    
    // std::fs::copy(&ctx.index_file_official, &ctx.index_file_internal).unwrap();
    
    0
}

fn test_combined_package(mut reader: TarReader, meta_group: VersionMetaGroup) {
    let mut data_locations = HashMap::<String, (u64, u64)>::new();
    let mut history = HistoryFile::new_dir("history_for_test", Weak::new());

    for meta in &meta_group {
        history.replay_operations(&meta);

        // 记录所有文件的数据和来源
        for change in &meta.changes {
            match change {
                FileChange::UpdateFile { path, offset, len, .. } => {
                    data_locations.insert(path.to_owned(), (*offset, *len));
                },
                FileChange::DeleteFile { path } => {
                    data_locations.remove(path);
                },
                FileChange::MoveFile { from, to } => {
                    let hold = data_locations.remove(from).unwrap();
                    data_locations.insert(to.to_owned(), hold);
                }
                _ => (),
            }
        }
    }
    
    let empty = HistoryFile::new_dir("empty_root", Weak::new());
    let diff = Diff::diff(&history, &empty, None);

    for up in diff.updated_files {
        let path = up.path();
        let path = path.deref();
        let (offset, len) = data_locations.get(path).unwrap();

        println!("正在测试: {} at {}+{}", path, offset, len);

        let mut open = reader.open_file(*offset, *len);
        let actual = calculate_hash(&mut open);
        let expected = up.hash();
        let expected = expected.deref();
        assert!(&actual == expected, "hashes do not match, path: {} actual: {}, expected: {}", path, actual, expected);
    }
}