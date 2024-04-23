//! 历史文件对象（文件状态快照）

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;
use std::rc::Weak;
use std::time::SystemTime;

use mcpatch_shared::data::version_meta::FileChange;
use mcpatch_shared::data::version_meta::VersionMeta;

use crate::diff::abstract_file::calculate_path_helper;
use crate::diff::abstract_file::find_file_helper;
use crate::diff::abstract_file::walk_abstract_file;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::abstract_file::BorrowIntoIterator;

/// 借用子文件列表
pub struct IntoIter<'a>(std::cell::Ref<'a, HashMap<String, HistoryFile>>);

impl BorrowIntoIterator for IntoIter<'_> {
    type Item = HistoryFile;

    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        self.0.values().map(|f| f.to_owned())
    }
}

/// 代表一个HistoryFile的实际数据部分
pub struct Inner {
    /// 父文件
    parent: RefCell<Weak<Inner>>,

    /// 文件名
    name: RefCell<String>,

    /// 文件长度
    len: u64,

    /// 文件修改时间
    modified: SystemTime,

    /// 是不是一个目录
    is_dir: bool,

    /// 文件的相对路径
    path: RefCell<String>,

    /// 文件的哈希值
    hash: String,
    
    /// 子文件列表
    children: RefCell<HashMap<String, HistoryFile>>,
}

/// 代表一个历史的文件状态，主要用于和目前磁盘上的文件状态对比计算文件差异
#[derive(Clone)]
pub struct HistoryFile(Rc<Inner>);

impl HistoryFile {
    /// 创建一个文件对象
    pub fn new_file(name: &str, modified: SystemTime, len: u64, hash: String, parent: Weak<Inner>) -> Self {
        let strong_parent = parent.clone().upgrade().map(|p| HistoryFile(p));
        
        Self(Rc::new(Inner {
            parent: RefCell::new(parent),
            name: RefCell::new(name.to_owned()),
            len,
            modified,
            is_dir: false,
            path: RefCell::new(calculate_path_helper(name, strong_parent.as_ref())),
            hash,
            children: RefCell::new(HashMap::new()),
        }))
    }

    /// 创建一个目录对象
    pub fn new_dir(name: &str, parent: Weak<Inner>) -> Self {
        let strong_parent = parent.clone().upgrade().map(|p| HistoryFile(p));
        
        Self(Rc::new(Inner {
            parent: RefCell::new(parent),
            name: RefCell::new(name.to_owned()),
            len: 0,
            modified: std::time::UNIX_EPOCH,
            is_dir: true,
            path: RefCell::new(calculate_path_helper(name, strong_parent.as_ref())),
            hash: "it is a dir".to_owned(),
            children: RefCell::new(HashMap::new()),
        }))
    }

    /// 创建一个空目录
    pub fn new_empty() -> Self {
        HistoryFile::new_dir("empty_root", Weak::new())
    }

    /// 复现`meta`上的所有文件操作
    pub fn replay_operations(&mut self, meta: &VersionMeta) {
        for change in &meta.changes {
            match change {
                FileChange::CreateFolder { path } =>  self.create_directory(&path),
                FileChange::UpdateFile { path, hash, len, modified, .. } => self.update_file(&path, hash, len, modified),
                FileChange::DeleteFolder { path } => self.delete_file_or_directory(&path),
                FileChange::DeleteFile { path } => self.delete_file_or_directory(&path),
                FileChange::MoveFile { from, to } => self.move_file(&from, &to),
            }
        }
    }

    /// 复现一个“更新文件”的操作
    pub fn update_file(&self, path: &str, hash: &String, len: &u64, modified: &SystemTime) {
        let (parent, end) = self.lookup_parent_and_end(path);

        let file = HistoryFile::new_file(end, *modified, *len, hash.to_owned(), Rc::downgrade(&parent));

        parent.children.borrow_mut().insert(end.to_owned(), file);
    }

    /// 复现一个“创建目录”的操
    pub fn create_directory(&self, path: &str) {
        let (parent, end) = self.lookup_parent_and_end(path);
        
        let dir = HistoryFile::new_dir(end, Rc::downgrade(&parent));

        parent.children.borrow_mut().insert(end.to_owned(), dir);
    }

    /// 复现一个“移动文件”的操作
    pub fn move_file(&self, from: &str, to: &str) {
        let (parent, end) = self.lookup_parent_and_end(from);

        // 从旧目录中拿起
        let holding = parent.children.borrow_mut().remove(end).unwrap();

        let (parent, end) = self.lookup_parent_and_end(to);

        // 修改文件名并从新计算路径
        *holding.name.borrow_mut() = end.to_owned();
        *holding.parent.borrow_mut() = Rc::downgrade(&parent);
        holding.recalculate_path();

        // 放到新目录下
        parent.children.borrow_mut().insert(end.to_owned(), holding);
    }

    /// 复现一个“删除文件”或者“删除目录”的操作
    pub fn delete_file_or_directory(&self, path: &str) {
        let (parent, end) = self.lookup_parent_and_end(path);
        
        let holding = parent.children.borrow_mut().remove(end).unwrap();

        assert!(holding.children.borrow().is_empty());
    }

    /// 查找一个文件
    fn lookup_parent_and_end<'a, 'b>(&'a self, path: &'b str) -> (HistoryFile, &'b str) {
        let (parent, end) = if path.contains("/") { 
            let (p, e) = path.rsplit_once("/").unwrap();

            (Some(p), e)
        } else { 
            (None, path) 
        };

        match parent {
            Some(parent) => (self.find(parent)
                .expect(&format!("can not found {} in {}", path, self.path().deref())), end),
            None => (self.clone(), end),
        }
    }

    /// 重新计算相对路径，一般文件被移动后要重新计算相对路径
    fn recalculate_path(&self) {
        let name = self.name.borrow();
        let mut path = self.path.borrow_mut();

        let parent = HistoryFile(self.parent.borrow().upgrade().unwrap());
        let new_path = calculate_path_helper(&name, Some(&parent));

        *path = new_path;
    }
}

impl Deref for HistoryFile {
    type Target = Rc<Inner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AbstractFile for HistoryFile {
    fn parent(&self) -> Option<HistoryFile> {
        self.parent.borrow().upgrade().map(|f| HistoryFile(f))
    }

    fn name(&self) -> impl Deref<Target = String> {
        self.name.borrow()
    }

    fn hash(&self) -> impl Deref<Target = String> {
        &self.hash
    }

    fn len(&self) -> u64 { 
        self.len
    }

    fn modified(&self) -> SystemTime {
        self.modified
    }

    fn is_dir(&self) -> bool {
        self.is_dir
    }

    fn path(&self) -> impl Deref<Target = String> {
        self.path.borrow()
    }

    fn files(&self) -> impl BorrowIntoIterator<Item = Self> {
        IntoIter(self.children.borrow())
    }

    fn find(&self, path: &str) -> Option<Self> {
        find_file_helper(self, path)
    }
}

impl Debug for HistoryFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&walk_abstract_file(self, 4))
    }
}
