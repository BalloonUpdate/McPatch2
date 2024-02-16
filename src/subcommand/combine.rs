use std::collections::HashMap;
use std::rc::Weak;

use crate::common::tar_reader::TarReader;
use crate::common::tar_writer::TarWriter;
use crate::data::index_file::IndexFile;
use crate::data::version_meta::FileChange;
use crate::diff::diff::Diff;
use crate::diff::history_file::HistoryFile;
use crate::AppContext;

pub const COMBINED_FILENAME: &str = "_combined.tar";

pub fn do_combine(ctx: AppContext) -> i32 {
    let index_file = IndexFile::load(&ctx.index_file_official);

    if index_file.into_iter().all(|e| e.filename != COMBINED_FILENAME) {
        println!("no data can be combined");
        return 0;
    }

    let mut history_file = HistoryFile::new_dir("workspace_root", Weak::new());
    let mut binary_data_refs = HashMap::<String, String>::new();

    // 读取现有更新包，并复现在history_file上
    for v in &index_file {
        let mut reader = TarReader::new(ctx.public_dir.join(&v.filename));
        for meta in &reader.read_metadata_group(v.offset, v.len) {
            history_file.replay_operations(&meta);

            // 记录所有文件的数据和来源
            for change in &meta.changes {
                match change {
                    FileChange::UpdateFile { path, .. } => {
                        binary_data_refs.insert(path.to_owned(), v.label.clone());
                    },
                    FileChange::DeleteFile { path } => {
                        binary_data_refs.remove(path);
                    },
                    _ => (),
                }
            }
        }
    }

    // 生成diff对象
    let empty = HistoryFile::new_dir("empty_root", Weak::new());
    let diff = Diff::diff(&history_file, &empty, None);

    // 生成新的合并包
    let new_tar_file = ctx.public_dir.join("_combined_temp.tar");
    let mut writer = TarWriter::new(&new_tar_file);






    std::fs::copy(&ctx.index_file_official, &ctx.index_file_internal).unwrap();
    
    0
}