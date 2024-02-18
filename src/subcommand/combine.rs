use std::collections::HashMap;
use std::rc::Weak;

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

/// 代表一个压缩包中的文件数据所在的位置
struct Location {
    /// 所在的tar包的文件名
    pub filename: String,

    /// tar包中的文件偏移
    pub offset: u64,

    /// 数据的长度
    pub len: u64,
}

pub fn do_combine(ctx: AppContext) -> i32 {
    let index_file = IndexFile::load(&ctx.index_file_internal);

    // // 收集所有要合并的版本
    // let versions_to_be_combined = index_file.into_iter()
    //     .filter(|e| e.filename != COMBINED_FILENAME)
    //     .collect::<LinkedList<_>>();

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
                            filename: v.filename.to_owned(),
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

    // 生成diff对象
    let empty = HistoryFile::new_dir("empty_root", Weak::new());
    let diff = Diff::diff(&history_file, &empty, None);

    // 生成新的合并包
    let new_tar_file = ctx.public_dir.join("_combined.temp.tar");
    let mut writer = TarWriter::new(&new_tar_file);

    // for (k, v) in &data_locations {
    //     println!("{k} => {},{},{}", v.filename, v.offset, v.len);
    // }

    // 写入每个版本里的所有文件数据
    for f in &diff.updated_files {
        let path = f.path().to_owned();
        // println!("get {path}");
        let loc = data_locations.get(&path).unwrap();
        let version_tar = ctx.public_dir.join(&loc.filename);

        // 读取tar包，直接跳转到对应位置
        let mut reader = TarReader::new(version_tar);
        let read = reader.open_file(loc.offset, loc.len);

        writer.write_file(read, f.len(), &path);
    }

    // 写入元数据
    let meta_offset = writer.finish(meta_group);

    // 更新索引文件
    let new_index_filepath = ctx.public_dir.join("_index.temp.json");
    let mut new_index_file = IndexFile::load(&new_index_filepath);
    for index in &index_file {
        new_index_file.add_index(VersionIndex {
            label: index.label.to_owned(),
            filename: new_tar_file.filename().to_owned(),
            offset: meta_offset.offset,
            len: meta_offset.length,
            hash: "no hash".to_owned(),
        })
    }
    new_index_file.save(&new_index_filepath);

    // 测试合并包
    /*
    for v in &new_index_file {
        let mut reader = TarReader::new(ctx.public_dir.join(&v.filename));
        let mut history = HistoryFile::new_dir("history_test", Weak::new());
        let group = reader.read_metadata_group(v.offset, v.len);
        let mut data_locations = HashMap::<String, Location>::new();

        for meta in &group {
            history.replay_operations(&meta);

            // 记录所有文件的数据和来源
            for change in &meta.changes {
                match change {
                    FileChange::UpdateFile { path, offset, len, .. } => {
                        data_locations.insert(path.to_owned(), Location {
                            filename: v.filename.to_owned(),
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
        let empty = HistoryFile::new_dir("empty_root", Weak::new());
        let diff = Diff::diff(&history_file, &empty, None);

        for up in diff.updated_files {
            group.find_meta()
        }

        for change in &meta.changes {
            if let FileChange::UpdateFile { path, hash, offset, len, .. } = change {
                println!("正在测试: {}({}) 的 {}", meta.label, v.filename, path);

                let mut open = reader.open_file(*offset, *len);
                let actual = calculate_hash(&mut open);
                assert!(&actual == hash, "hashes do not match, path: {} actual: {}, expected: {}", path, actual, hash);
            }
        }
        
    }
    */

    // std::fs::copy(&ctx.index_file_official, &ctx.index_file_internal).unwrap();
    
    0
}