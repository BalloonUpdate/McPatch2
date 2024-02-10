use std::collections::LinkedList;
use std::ops::Deref;
use std::time::SystemTime;

pub trait BorrowIntoIterator {
    type Item;

    fn iter(&self) -> impl Iterator<Item = Self::Item>;
}

pub trait AbstractFile : Clone {
    fn parent(&self) -> Option<Self>;

    fn name(&self) -> impl Deref<Target = String>;
    
    fn hash(&self) -> impl Deref<Target = String>;
    
    fn len(&self) -> u64;

    fn modified(&self) -> SystemTime;
    
    fn is_dir(&self) -> bool;
    
    fn path(&self) -> impl Deref<Target = String>;
    
    fn files(&self) -> impl BorrowIntoIterator<Item = Self>;

    fn find(&self, path: &str) -> Option<Self>;
}

pub fn find_file_helper<T: AbstractFile>(parent: &T, path: &str) -> Option<T> {
    assert!(parent.is_dir());
    assert!(!path.contains("\\"));

    let mut result = parent.to_owned();

    for frag in path.split("/") {
        let found = result.files().iter().find(|f| f.name().deref() == frag);

        match found {
            Some(found) => result = found,
            None => return None,
        }
    }

    return Some(result);
}

pub fn calculate_path_helper(name: &str, parent: Option<&impl AbstractFile>) -> String {
    match parent {
        Some(parent) => {
            let parent_path = parent.path();
            let parent_path = parent_path.deref();

            if parent_path.starts_with(":") {
                name.to_owned()
            } else {
                format!("{}/{}", parent_path, name)
            }
        },
        None => format!(":{}:", name),
    }
}

pub fn abstract_file_to_string(f: &impl AbstractFile) -> String {
    if f.is_dir() {
        format!("{} (directory: {}) {}", &f.name().deref(), f.files().iter().count(), f.path().deref())
    } else {
        let dt = chrono::DateTime::<chrono::Local>::from(f.modified().to_owned());

        format!("{} ({}, {}, {}) {}", &f.name().deref(), f.len(), f.hash().deref(), dt.format("%Y-%m-%d %H:%M:%S"), f.path().deref())
    }
}

pub fn walk_abstract_file(file: &impl AbstractFile, indent: usize) -> String {
    let mut buf = String::with_capacity(1024);
    let mut stack = LinkedList::new();

    stack.push_back((file.to_owned(), 0));

    while let Some(pop) = stack.pop_back() {
        let (f, depth) = pop;

        for _ in 0..depth * indent {
            buf += " ";
        }

        buf += &format!("{}\n", abstract_file_to_string(&f));

        if f.is_dir() {
            for ff in f.files().iter() {
                stack.push_back((ff.to_owned(), depth + 1));
            }
        }
    }

    buf
}