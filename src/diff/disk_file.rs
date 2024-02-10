use std::cell::RefCell;
use std::collections::LinkedList;
use std::fmt::Debug;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::rc::Weak;
use std::time::SystemTime;

use crate::common::file_hash::calculate_hash;
use crate::diff::abstract_file::calculate_path_helper;
use crate::diff::abstract_file::find_file_helper;
use crate::diff::abstract_file::walk_abstract_file;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::abstract_file::BorrowIntoIterator;
use crate::utility::ext::GetFileNamePart;

pub struct BorrowedHash<'a>(std::cell::Ref<'a, Option<String>>);

impl Deref for BorrowedHash<'_> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

pub struct IntoIter<'a, T>(std::cell::Ref<'a, Option<LinkedList<T>>>);

impl BorrowIntoIterator for IntoIter<'_, DiskFile> {
    type Item = DiskFile;

    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        self.0.as_ref().unwrap().iter().map(|f| f.to_owned())
    }
}

pub struct Inner {
    file: PathBuf,
    parent: Option<Weak<Inner>>,
    name: String,
    len: u64,
    modified: SystemTime,
    is_dir: bool,
    path: RefCell<String>,
    hash: RefCell<Option<String>>,
    children: RefCell<Option<LinkedList<DiskFile>>>,
}

#[derive(Clone)]
pub struct DiskFile(Rc<Inner>);

impl Deref for DiskFile {
    type Target = Rc<Inner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DiskFile {
    pub fn new(path: PathBuf, parent: Option<Weak<Inner>>) -> Self {
        let filename = path.filename().to_owned();
        let metadata = std::fs::metadata(&path).unwrap();
        let strong_parent = parent.clone().and_then(|p| p.upgrade()).map(|p| DiskFile(p));

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

    pub fn disk_file(&self) -> &Path {
        &self.file
    }
}

impl AbstractFile for DiskFile {
    fn parent(&self) -> Option<DiskFile> {
        self.parent.as_ref().and_then(|f| f.upgrade()).map(|f| DiskFile(f))
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
                
                let child = DiskFile::new(file.path(), Some(Rc::downgrade(&self.0)));
                
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