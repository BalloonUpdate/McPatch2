//! 目录差异对比

use std::collections::LinkedList;
use std::fmt::Debug;
use std::fmt::Write;
use std::ops::Deref;
use std::fmt::Display;
use std::time::UNIX_EPOCH;

use crate::core::data::version_meta::FileChange;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::abstract_file::BorrowIntoIterator;
use crate::core::rule_filter::RuleFilter;

const OP_FULL_ADDED_FOLDER: &str = "创建目录: ";
const OP_FULL_ADDED_FILE: &str   = "更新文件: ";
const OP_FULL_MODIFIED_FILE: &str   = "修改文件: ";
const OP_FULL_MISSING_FOLDER: &str = "删除目录: ";
const OP_FULL_MISSING_FILE: &str   = "删除文件: ";
const OP_FULL_MOVE_FILE: &str     = "移动文件: ";
const OP_SHORT_ADDED_FOLDER: &str = OP_FULL_ADDED_FOLDER;
const OP_SHORT_ADDED_FILE: &str   = OP_FULL_ADDED_FILE;
const OP_SHORT_MODIFIED_FILE: &str   = OP_FULL_MODIFIED_FILE;
const OP_SHORT_MISSING_FOLDER: &str = OP_FULL_MISSING_FOLDER;
const OP_SHORT_MISSING_FILE: &str   = OP_FULL_MISSING_FILE;
const OP_SHORT_MOVE_FILE: &str     = OP_FULL_MOVE_FILE;

/// 代表一组文件差异
pub struct Diff<N: AbstractFile, O: AbstractFile> {
    pub added_folders: Vec<N>,
    pub added_files: Vec<N>,
    pub modified_files: Vec<N>,
    pub missing_folders: Vec<O>,
    pub missing_files: Vec<O>,
    pub renamed_files: Vec<(O, N)>,
    excluding_filter: RuleFilter,
}

impl<N: AbstractFile, O: AbstractFile> Diff<N, O> {
    /// 执行目录比较
    pub fn diff(newer: &N, older: &O, filter_rules: Option<&Vec<String>>) -> Self {
        let mut result = Diff {
            added_folders: Vec::new(),
            added_files: Vec::new(),
            modified_files: Vec::new(),
            missing_folders: Vec::new(),
            missing_files: Vec::new(),
            renamed_files: Vec::new(),
            excluding_filter: match filter_rules {
                Some(filter_rules) => RuleFilter::from_rules(filter_rules.iter()),
                None => RuleFilter::new(),
            },
        };

        result.find_added(newer, older);
        result.find_missing(newer, older);
        result.find_modified(newer, older);
        result.detect_file_movings(newer, older);

        result
    }

    /// 有没有不同
    pub fn has_diff(&self) -> bool {
        !self.added_folders.is_empty() ||
        !self.added_files.is_empty() ||
        !self.modified_files.is_empty() ||
        !self.missing_folders.is_empty() ||
        !self.missing_files.is_empty() ||
        !self.renamed_files.is_empty()
    }

    /// 寻找新增的文件
    fn find_added(&mut self, newer: &N, older: &O) {
        assert!(newer.is_dir());
        assert!(older.is_dir());

        for n in newer.files().iter() {
            if !self.is_visible(n.path().deref()) {
                continue;
            }

            let find = older.find(&n.name());

            match find {
                Some(o) => {
                    match (n.is_dir(), o.is_dir()) {
                        // 两边都是目录则进入递归
                        (true, true) => self.find_added(&n, &o),

                        // 两边类型不一样，则会先删除后添加
                        (true, false) => self.mark_as_added(&n),
                        (false, true) => self.mark_as_added(&n),

                        // 两边都是文件，跳过，会由文件修改检查函数来处理此情况
                        (false, false) => (),
                    }
                },

                // 在旧目录里找不到，此时肯定是新增的文件
                None => self.mark_as_added(&n),
            }
        }
    }

    /// 寻找删除的文件
    fn find_missing(&mut self, newer: &N, older: &O) {
        assert!(newer.is_dir());
        assert!(older.is_dir());

        for o in older.files().iter() {
            let found = match newer.find(&o.name()) {
                Some(o) => if self.is_visible(o.path().deref()) { Some(o) } else { None },
                None => None,
            };

            match found {
                Some(n) => match (o.is_dir(), n.is_dir()) {
                    // 两边都是目录就进入递归
                    (true, true) => self.find_missing(&n, &o),

                    // 两边文件类型不一样，就先删除再添加
                    (true, false) => self.mark_as_missing(&o),
                    (false, true) => self.mark_as_missing(&o),

                    // 两边都是文件，跳过，会由文件修改检查函数来处理此情况
                    (false, false) => (),
                },

                // 在新目录里找不到，此时肯定是被删除的文件
                None => self.mark_as_missing(&o),
            }
        }
    }

    /// 寻找修改的文件
    fn find_modified(&mut self, newer: &N, older: &O) {
        assert!(newer.is_dir());
        assert!(older.is_dir());

        for n in newer.files().iter() {
            if !self.is_visible(n.path().deref()) {
                continue;
            }

            let find = older.find(&n.name());

            match find {
                Some(o) => {
                    match (n.is_dir(), o.is_dir()) {
                        // 两边都是目录则进入递归
                        (true, true) => self.find_modified(&n, &o),

                        // 两边类型不一样，跳过，会由文件新增和文件删除检测函数来处理此情况
                        (true, false) => (),
                        (false, true) => (),

                        // 两边都是文件，则对比文件，如果不同，视为修改过的文件
                        (false, false) => if !self.compare_file(&n, &o) {
                            self.mark_as_modified(&n)
                        },
                    }
                },

                // 在旧目录里找不到，此情况已由文件新增检测函数处理过
                None => (),
            }
        }
    }

    /// 将一个文件或者目录标记成删除
    fn mark_as_missing(&mut self, file: &O) {
        if file.is_dir() {
            for f in file.files().iter() {
                self.mark_as_missing(&f);
            }

            self.missing_folders.push(file.to_owned());
        } else {
            self.missing_files.push(file.to_owned());
        }
    }

    /// 将一个文件或者目录标记为新增
    fn mark_as_added(&mut self, file: &N) {
        if !self.is_visible(&file.path()) {
            return;
        }

        if file.is_dir() {
            self.added_folders.push(file.clone());

            for f in file.files().iter() {
                self.mark_as_added(&f);
            }
        } else {
            self.added_files.push(file.to_owned());
        }
    }

    /// 将一个文件标记为修改过的文件，目录不行
    fn mark_as_modified(&mut self, file: &N) {
        if !self.is_visible(&file.path()) {
            return;
        }

        assert!(!file.is_dir());

        self.modified_files.push(file.to_owned());
    }
    
    /// 比较两个文件是否相同
    fn compare_file(&self, a: &N, b: &O) -> bool {
        let ta = a.modified().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let tb = b.modified().duration_since(UNIX_EPOCH).unwrap().as_secs();

        ta == tb || a.hash().deref() == b.hash().deref()
    }

    /// 检查一个文件要不要被忽略
    fn is_visible(&self, path: &str) -> bool {
        !self.excluding_filter.test_any(path, false)
    }
    
    /// 检测文件移动操作
    fn detect_file_movings(&mut self, newer: &N, older: &O) {
        // 首先收集所有可能的移动操作
        for updated in &self.added_files {
            for deleted in &self.missing_files {
                let n = newer.find(updated.path().deref()).unwrap();
                let o = older.find(deleted.path().deref()).unwrap();

                if n.modified() == o.modified() {
                    continue;
                }

                if n.hash().deref() == o.hash().deref() {
                    self.renamed_files.push((o, n));
                }
            }

            // println!("{}", updated.path().deref());
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
            for i in (0..self.added_files.len()).rev() {
                if self.added_files[i].path().deref() == moving.1.path().deref() {
                    self.added_files.remove(i);
                }
            }

            for i in (0..self.missing_files.len()).rev() {
                if self.missing_files[i].path().deref() == moving.0.path().deref() {
                    self.missing_files.remove(i);
                }
            }
        }
    }

    /// 将一个`diff`对象转换成文件变动列表
    pub fn to_file_changes(&self) -> LinkedList<FileChange> {
        let mut changes = LinkedList::new();
    
        for f in &self.missing_files {
            changes.push_back(FileChange::DeleteFile { 
                path: f.path().to_owned() 
            })
        }
    
        for f in &self.added_folders {
            changes.push_back(FileChange::CreateFolder { 
                path: f.path().to_owned() 
            })
        }
    
        for f in &self.renamed_files {
            changes.push_back(FileChange::MoveFile {
                from: f.0.path().to_owned(), 
                to: f.1.path().to_owned()
            })
        }
    
        for f in &self.added_files {
            changes.push_back(FileChange::UpdateFile { 
                path: f.path().to_owned(), 
                hash: f.hash().to_owned(), 
                len: f.len(), 
                modified: f.modified(), 
                offset: 0, // 此时offset是空的，需要由TarWriter去填充
            })
        }

        for f in &self.modified_files {
            changes.push_back(FileChange::UpdateFile { 
                path: f.path().to_owned(), 
                hash: f.hash().to_owned(), 
                len: f.len(), 
                modified: f.modified(), 
                offset: 0, // 此时offset是空的，需要由TarWriter去填充
            })
        }
    
        for f in &self.missing_folders {
            changes.push_back(FileChange::DeleteFolder { 
                path: f.path().to_owned() 
            })
        }
    
        changes
    }
}

impl<N: AbstractFile, O: AbstractFile> Display for Diff<N, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Diff ({}{}, {}{}, {}{}, {}{}, {}{}, {}{})",
            OP_SHORT_ADDED_FOLDER, self.added_folders.len(),
            OP_SHORT_ADDED_FILE, self.added_files.len(),
            OP_SHORT_MODIFIED_FILE, self.modified_files.len(),
            OP_SHORT_MISSING_FOLDER, self.missing_folders.len(),
            OP_SHORT_MISSING_FILE, self.missing_files.len(),
            OP_SHORT_MOVE_FILE, self.renamed_files.len(),
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

impl<N: AbstractFile, O: AbstractFile> Debug for Diff<N, O> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut need_newline = false;

        for f in &self.missing_files {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{}", OP_FULL_MISSING_FILE, f.path().deref()))?;
        }
    
        for f in &self.added_folders {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{}", OP_FULL_ADDED_FOLDER, f.path().deref()))?;
        }
    
        for (n, o) in &self.renamed_files {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{} -> {}", OP_FULL_MOVE_FILE, n.path().deref(), o.path().deref()))?;
        }
    
        for f in &self.added_files {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{}", OP_FULL_ADDED_FILE, f.path().deref()))?;
        }

        for f in &self.modified_files {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{}", OP_FULL_MODIFIED_FILE, f.path().deref()))?;
        }
    
        for f in &self.missing_folders {
            printn!(need_newline, fmt);
            fmt.write_str(&format!("{}{}", OP_FULL_MISSING_FOLDER, f.path().deref()))?;
        }

        Ok(())
    }
}