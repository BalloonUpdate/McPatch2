use std::fmt::Debug;
use std::fmt::Write;
use std::ops::Deref;
use std::fmt::Display;
use std::time::UNIX_EPOCH;

use crate::diff::abstract_file::AbstractFile;
use crate::diff::abstract_file::BorrowIntoIterator;
use crate::diff::rule_filter::RuleFilter;

const OP_FULL_CREATE_FOLDER: &str = "创建目录: ";
const OP_FULL_UPDATE_FILE: &str   = "更新文件: ";
const OP_FULL_DELETE_FOLDER: &str = "删除目录: ";
const OP_FULL_DELETE_FILE: &str   = "删除文件: ";
const OP_FULL_MOVE_FILE: &str     = "移动文件: ";
const OP_SHORT_CREATE_FOLDER: &str = OP_FULL_CREATE_FOLDER;
const OP_SHORT_UPDATE_FILE: &str   = OP_FULL_UPDATE_FILE;
const OP_SHORT_DELETE_FOLDER: &str = OP_FULL_DELETE_FOLDER;
const OP_SHORT_DELETE_FILE: &str   = OP_FULL_DELETE_FILE;
const OP_SHORT_MOVE_FILE: &str     = OP_FULL_MOVE_FILE;

/// 代表一组文件差异
pub struct Diff<'a, N: AbstractFile, O: AbstractFile> {
    pub created_folders: Vec<N>,
    pub updated_files: Vec<N>,
    pub deleted_folders: Vec<O>,
    pub deleted_files: Vec<O>,
    pub renamed_files: Vec<(O, N)>,
    filter: &'a RuleFilter,
}

impl<'a, N: AbstractFile, O: AbstractFile> Diff<'a, N, O> {
    /// 执行目录比较
    pub fn diff(newer: &N, older: &O, filter: &'a RuleFilter) -> Self {
        let mut result = Diff {
            created_folders: Vec::new(),
            updated_files: Vec::new(),
            deleted_folders: Vec::new(),
            deleted_files: Vec::new(),
            renamed_files: Vec::new(),
            filter,
        };

        result.find_deleteds(newer, older);
        result.find_updateds(newer, older);
        result.detect_file_movings(newer, older);

        result
    }

    /// 有没有不同
    pub fn has_diff(&self) -> bool {
        !self.created_folders.is_empty() ||
        !self.updated_files.is_empty() ||
        !self.deleted_folders.is_empty() ||
        !self.deleted_files.is_empty() ||
        !self.renamed_files.is_empty()
    }

    /// 寻找已经删除的文件
    fn find_deleteds(&mut self, newer: &N, older: &O) {
        assert!(newer.is_dir());
        assert!(older.is_dir());

        // let filter = |f: Option<N>| f.and_then(|f| if self.filter(f.path().deref()) { Some(f) } else { None });
        
        for o in older.files().iter() {
            let found = match newer.find(&o.name()) {
                Some(o) => if self.filter(o.path().deref()) { Some(o) } else { None },
                None => None,
            };

            match found {
                Some(n) => match (o.is_dir(), n.is_dir()) {
                    (true, true) => self.find_deleteds(&n, &o),
                    (true, false) => self.mark_as_deleted(&o),
                    (false, true) => self.mark_as_deleted(&o),
                    (false, false) => if !self.compare_file(&n, &o) {
                        self.mark_as_updated(&n)
                    },
                },
                None => {
                    self.mark_as_deleted(&o);
                    continue;
                },
            }
        }
    }

    /// 寻找新增或者修改的文件
    fn find_updateds(&mut self, newer: &N, older: &O) {
        assert!(newer.is_dir());
        assert!(older.is_dir());

        for n in newer.files().iter() {
            if !self.filter(n.path().deref()) {
                continue;
            }

            let find = older.find(&n.name());

            match find {
                Some(o) => if n.is_dir() && o.is_dir() {
                    self.find_updateds(&n, &o);
                },
                None => self.mark_as_updated(&n),
            };
        }
    }

    /// 将一个文件标记成已经删除的文件
    fn mark_as_deleted(&mut self, file: &O) {
        if file.is_dir() {
            for f in file.files().iter() {
                self.mark_as_deleted(&f);
            }

            self.deleted_folders.push(file.to_owned());
        } else {
            self.deleted_files.push(file.to_owned());
        }
    }

    /// 将一个文件标记为新增或者修改过的文件
    fn mark_as_updated(&mut self, file: &N) {
        if file.is_dir() {
            self.created_folders.push(file.clone());

            for f in file.files().iter() {
                self.mark_as_updated(&f);
            }
        } else {
            self.updated_files.push(file.to_owned());
        }
    }
    
    /// 比较两个文件是否相同
    fn compare_file(&self, a: &N, b: &O) -> bool {
        let ta = a.modified().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let tb = b.modified().duration_since(UNIX_EPOCH).unwrap().as_secs();

        ta == tb || a.hash().deref() == b.hash().deref()
    }

    /// 检查一个文件要不要被忽略
    fn filter(&self, path: &str) -> bool {
        self.filter.test_all(path, true)
    }
    
    /// 检测文件移动操作
    fn detect_file_movings(&mut self, newer: &N, older: &O) {
        // 首先收集所有可能的移动操作
        for updated in &self.updated_files {
            for deleted in &self.deleted_files {
                let n = newer.find(updated.path().deref()).unwrap();
                let o = older.find(deleted.path().deref()).unwrap();

                if n.hash().deref() == o.hash().deref() {
                    self.renamed_files.push((o, n));
                }
            }
        }

        // 如果有多个同名但不同路径的文件移动，就将它们退回复制操作，而非移动
        let mut ambiguous = Vec::<String>::new();
        let mut scanned = Vec::<String>::new();

        for moving in &self.renamed_files {
            if scanned.contains(&moving.0.path().to_owned()) {
                if !ambiguous.contains(&moving.0.path().to_owned()) {
                    ambiguous.push(moving.0.path().to_owned());
                }
            } else {
                scanned.push(moving.0.path().to_owned());
            }
        }

        for a in ambiguous {
            for i in (0..self.renamed_files.len()).rev() {
                if self.renamed_files[i].0.path().deref() == &a {
                    self.renamed_files.remove(i);
                }
            }
        }

        // 将复制操作简化为移动操作
        for moving in &self.renamed_files {
            for i in (0..self.updated_files.len()).rev() {
                if self.updated_files[i].path().deref() == moving.1.path().deref() {
                    self.updated_files.remove(i);
                }
            }

            for i in (0..self.deleted_files.len()).rev() {
                if self.deleted_files[i].path().deref() == moving.0.path().deref() {
                    self.deleted_files.remove(i);
                }
            }
        }
    }
}

impl<N: AbstractFile, O: AbstractFile> Display for Diff<'_, N, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Diff ({}{}, {}{}, {}{}, {}{}, {}{})",
            OP_SHORT_CREATE_FOLDER,
            self.created_folders.len(),
            OP_SHORT_UPDATE_FILE,
            self.updated_files.len(),
            OP_SHORT_DELETE_FOLDER,
            self.deleted_folders.len(),
            OP_SHORT_DELETE_FILE,
            self.deleted_files.len(),
            OP_SHORT_MOVE_FILE,
            self.renamed_files.len(),
        ))
    }
}

macro_rules! printn {
    ($flag:ident, $fmt:ident) => {
        if $flag {
            $fmt.write_char('\n')?;
        }

        $flag = true;
    };
}

impl<N: AbstractFile, O: AbstractFile> Debug for Diff<'_, N, O> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut need_newline = false;

        for f in &self.deleted_files {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{}", OP_FULL_DELETE_FILE, f.path().deref()))?;
        }
    
        for f in &self.created_folders {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{}", OP_FULL_CREATE_FOLDER, f.path().deref()))?;
        }
    
        for (n, o) in &self.renamed_files {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{} -> {}", OP_FULL_MOVE_FILE, n.path().deref(), o.path().deref()))?;
        }
    
        for f in &self.updated_files {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{}", OP_FULL_UPDATE_FILE, f.path().deref()))?;
        }
    
        for f in &self.deleted_folders {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{}", OP_FULL_DELETE_FOLDER, f.path().deref()))?;
        }

        Ok(())
    }
}