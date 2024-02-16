use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;
use std::rc::Weak;
use std::time::SystemTime;

use crate::data::version_meta::FileChange;
use crate::data::version_meta::VersionMeta;
use crate::diff::abstract_file::calculate_path_helper;
use crate::diff::abstract_file::find_file_helper;
use crate::diff::abstract_file::walk_abstract_file;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::abstract_file::BorrowIntoIterator;

pub struct IntoIter<'a>(std::cell::Ref<'a, HashMap<String, HistoryFile>>);

impl BorrowIntoIterator for IntoIter<'_> {
    type Item = HistoryFile;

    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        self.0.values().map(|f| f.to_owned())
    }
}

pub struct Inner {
    parent: RefCell<Weak<Inner>>,
    name: RefCell<String>,
    len: u64,
    modified: SystemTime,
    is_dir: bool,
    path: RefCell<String>,
    hash: String,
    children: RefCell<HashMap<String, HistoryFile>>,
}

#[derive(Clone)]
pub struct HistoryFile(Rc<Inner>);

impl HistoryFile {
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

    pub fn update_file(&self, path: &str, hash: &String, len: &u64, modified: &SystemTime) {
        let (parent, end) = self.lookup_parent_and_end(path);

        let file = HistoryFile::new_file(end, *modified, *len, hash.to_owned(), Rc::downgrade(&parent));

        parent.children.borrow_mut().insert(end.to_owned(), file);
    }

    pub fn create_directory(&self, path: &str) {
        let (parent, end) = self.lookup_parent_and_end(path);
        
        let dir = HistoryFile::new_dir(end, Rc::downgrade(&parent));

        parent.children.borrow_mut().insert(end.to_owned(), dir);
    }

    pub fn move_file(&self, from: &str, to: &str) {
        let (parent, end) = self.lookup_parent_and_end(from);

        // 从旧目录总拿起
        let holding = parent.children.borrow_mut().remove(end).unwrap();

        let (parent, end) = self.lookup_parent_and_end(to);

        // 修改文件名并从新计算路径
        *holding.name.borrow_mut() = end.to_owned();
        *holding.parent.borrow_mut() = Rc::downgrade(&parent);
        holding.recalculate_path();

        // 放到新目录下
        parent.children.borrow_mut().insert(end.to_owned(), holding);
    }

    pub fn delete_file_or_directory(&self, path: &str) {
        let (parent, end) = self.lookup_parent_and_end(path);
        
        let holding = parent.children.borrow_mut().remove(end).unwrap();

        assert!(holding.children.borrow().is_empty());
    }

    fn lookup_parent_and_end<'a, 'b>(&'a self, path: &'b str) -> (HistoryFile, &'b str) {
        let (parent, end) = if path.contains("/") { 
            let (p, e) = path.rsplit_once("/").unwrap();

            (Some(p), e)
        } else { 
            (None, path) 
        };

        match parent {
            Some(parent) => (self.find(parent).expect(&format!("found {} in {}", path, self.path().deref())), end),
            None => (self.clone(), end),
        }
    }

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
