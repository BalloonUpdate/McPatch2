//! 更新包解压测试

use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;

use crate::core::data::version_meta::FileChange;
use crate::core::file_hash::calculate_hash;
use crate::core::tar_reader::TarReader;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::history_file::HistoryFile;

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
    pub fn finish<F: FnMut(Testing) -> ()>(mut self, mut f: F) -> Result<(), Failure> {
        self.finished = true;

        let empty = HistoryFile::new_empty();
        let diff = Diff::diff(&self.history, &empty, None);

        let mut vec = Vec::<&HistoryFile>::new();

        for f in &diff.added_files {
            vec.push(f);
        }

        for f in &diff.modified_files {
            vec.push(f);
        }

        let total = vec.len();

        for (index, up) in vec.iter().enumerate() {
            let path = up.path();
            let path = path.deref();
            let (archive, offset, len, label) = self.file_locations.get(path).unwrap();

            // println!("{index}/{total} 正在测试 {label} 的 {path} ({offset}+{len})");
            f(Testing { index, total, label, path, offset: *offset, len: *len });

            let mut reader = TarReader::new(&archive);
            let mut open = reader.open_file(*offset, *len);
            let actual = calculate_hash(&mut open);
            let expected = up.hash();
            let expected = expected.deref();

            if &actual != expected {
                return Err(Failure {
                    path: path.to_owned(), 
                    label: label.to_owned(), 
                    actual, 
                    expected: expected.to_owned(),
                });
            }

            // assert!(
            //     &actual == expected, 
            //     "文件哈希不匹配！文件路径: {}, 版本: {} 实际: {}, 预期: {}, 偏移: 0x{offset:x}, 长度: {len}",
            //     path, label, actual, expected
            // );
        }

        Ok(())
    }
}

impl Drop for ArchiveTester {
    fn drop(&mut self) {
        assert!(self.finished, "ArchiveTester is not finished yet!")
    }
}

/// 代表测试过程中的日志
#[derive(Debug)]
pub struct Testing<'a> {
    /// 当前正在测试第几个文件
    pub index: usize,

    /// 一共有多少个文件
    pub total: usize,

    /// 正在测试的文件所属的版本标签
    pub label: &'a str,

    /// 正在测试的文件在更新包里的相对路径
    pub path: &'a str,

    /// 正在测试的文件在更新包里的偏移地址
    pub offset: u64,

    /// 正在测试的文件在更新包里的大小
    pub len: u64,
}

/// 代表一个更新包测试的失败结果
#[derive(Debug)]
pub struct Failure {
    /// 失败的文件的路径
    pub path: String,

    /// 失败的文件所属的版本标签
    pub label: String,

    /// 实际的校验值
    pub actual: String,
    
    /// 预期的校验值
    pub expected: String,
}


