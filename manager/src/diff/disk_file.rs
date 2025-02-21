//! 磁盘文件对象

use std::cell::RefCell;
use std::collections::LinkedList;
use std::fmt::Debug;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::rc::Weak;
use std::time::SystemTime;

use crate::core::file_hash::calculate_hash;
use crate::diff::abstract_file::calculate_path_helper;
use crate::diff::abstract_file::find_file_helper;
use crate::diff::abstract_file::walk_abstract_file;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::abstract_file::BorrowIntoIterator;
use crate::utility::filename_ext::GetFileNamePart;

/// 借用哈希
pub struct BorrowedHash<'a>(std::cell::Ref<'a, Option<String>>);

impl Deref for BorrowedHash<'_> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

/// 借用子文件列表
pub struct IntoIter<'a>(std::cell::Ref<'a, Option<LinkedList<DiskFile>>>);

impl BorrowIntoIterator for IntoIter<'_> {
    type Item = DiskFile;

    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        self.0.as_ref().unwrap().iter().map(|f| f.to_owned())
    }
}

/// 代表一个DiskFile的实际数据部分
pub struct Inner {
    /// 文件在磁盘上的绝对路径
    file: PathBuf,

    /// 父文件
    parent: Weak<Inner>,

    /// 文件名
    name: String,

    /// 文件长度
    len: u64,

    /// 文件修改时间
    modified: SystemTime,

    /// 是不是一个目录
    is_dir: bool,

    /// 文件的相对路径
    path: RefCell<String>,

    /// 文件的哈希值缓存
    hash: RefCell<Option<String>>,

    /// 子文件列表缓存
    children: RefCell<Option<LinkedList<DiskFile>>>,
}

/// 代表目前磁盘上的文件状态，主要用于和历史状态对比计算文件差异
#[derive(Clone)]
pub struct DiskFile(Rc<Inner>);

impl Deref for DiskFile {
    type Target = Rc<Inner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DiskFile {
    /// 从磁盘路径创建
    pub fn new(path: PathBuf, parent: Weak<Inner>) -> Self {
        let filename = path.filename().to_owned();
        let metadata = std::fs::metadata(&path).unwrap();
        let strong_parent = parent.clone().upgrade().map(|p| DiskFile(p));

        let inner = Inner {
            file: path, 
            parent,
            name: filename.to_owned(), 
            len: metadata.len(), 
            modified: metadata.modified().unwrap(), 
            is_dir: metadata.is_dir(), 
            path: RefCell::new(calculate_path_helper(&filename, strong_parent.as_ref())), 
            hash: RefCell::new(None), 
            children: RefCell::new(None), 
        };

        Self(Rc::new(inner))
    }

    /// 返回磁盘路径的引用
    pub fn disk_file(&self) -> &Path {
        &self.file
    }
}

impl AbstractFile for DiskFile {
    fn parent(&self) -> Option<DiskFile> {
        self.parent.upgrade().map(|f| DiskFile(f))
    }

    fn name(&self) -> impl Deref<Target = String> {
        &self.name
    }

    fn hash(&self) -> impl Deref<Target = String> {
        assert!(!self.is_dir);
        
        let mut hash_mut = self.hash.borrow_mut();

        if hash_mut.is_none() {
            let mut fd = std::fs::File::open(&self.file).unwrap();
            *hash_mut = Some(calculate_hash(&mut fd));
        }

        drop(hash_mut);

        BorrowedHash(self.hash.borrow())
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
        assert!(self.is_dir);

        let mut children_mut = self.children.borrow_mut();

        if children_mut.is_none() {
            let mut result = LinkedList::new();
        
            for file in std::fs::read_dir(&self.file).unwrap() {
                let file = file.unwrap();
                
                let child = DiskFile::new(file.path(), Rc::downgrade(&self.0));
                
                result.push_back(child);
            }

            *children_mut = Some(result);
        }

        drop(children_mut);

        IntoIter(self.children.borrow())
    }
    
    fn find(&self, path: &str) -> Option<Self> {
        find_file_helper(self, path)
    }
}

impl Debug for DiskFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&walk_abstract_file(self, 4))
    }
}