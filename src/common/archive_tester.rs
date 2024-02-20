//! 更新包解压测试

use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;

use crate::common::file_hash::calculate_hash;
use crate::common::tar_reader::TarReader;
use crate::data::version_meta::FileChange;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::history_file::HistoryFile;
use crate::utility::extension::filename::GetFileNamePart;

pub struct ArchiveTester {
    /// key: 文件路径，value: (更新包路径, 偏移值, 长度, 版本号)
    file_locations: HashMap<String, (PathBuf, u64, u64, String)>,

    /// 当前文件状态
    history: HistoryFile,

    finished: bool,
}

impl ArchiveTester {
    pub fn new() -> Self {
        Self {
            file_locations: HashMap::new(), 
            history: HistoryFile::new_empty(),
            finished: false,
        }
    }

    /// 添加一个待测文件
    pub fn feed(&mut self, archive: impl AsRef<Path>, meta_offset: u64, meta_len: u32) {
        let mut reader = TarReader::new(&archive);
        let meta_group = reader.read_metadata_group(meta_offset, meta_len);

        for meta in &meta_group {
            self.history.replay_operations(&meta);
    
            // 记录所有文件的数据和来源
            for change in &meta.changes {
                match change {
                    FileChange::UpdateFile { path, offset, len, .. } => {
                        let tuple = (archive.as_ref().to_owned(), *offset, *len, meta.label.to_owned());
                        self.file_locations.insert(path.to_owned(), tuple);
                    },
                    FileChange::DeleteFile { path } => {
                        self.file_locations.remove(path);
                    },
                    FileChange::MoveFile { from, to } => {
                        let hold = self.file_locations.remove(from).unwrap();
                        self.file_locations.insert(to.to_owned(), hold);
                    }
                    _ => (),
                }
            }
        }
    }

    /// 开始测试
    pub fn finish(mut self) {
        self.finished = true;

        let empty = HistoryFile::new_empty();
        let diff = Diff::diff(&self.history, &empty, None);

        for up in diff.updated_files {
            let path = up.path();
            let path = path.deref();
            let (archive, offset, len, label) = self.file_locations.get(path).unwrap();
            let filename = archive.filename();

            println!("正在测试 {label} 的 {path} ({offset}+{len})");

            let mut reader = TarReader::new(&archive);
            let mut open = reader.open_file(*offset, *len);
            let actual = calculate_hash(&mut open);
            let expected = up.hash();
            let expected = expected.deref();

            assert!(
                &actual == expected, 
                "文件哈希不匹配！文件名: {}, 版本: {} 实际: {}, 预期: {}",
                filename, label, actual, expected
            );
        }
    }
}

impl Drop for ArchiveTester {
    fn drop(&mut self) {
        assert!(self.finished, "ArchiveTester is not finished yet!")
    }
}