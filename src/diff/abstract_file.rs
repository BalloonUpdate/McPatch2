use std::collections::LinkedList;
use std::ops::Deref;
use std::time::SystemTime;

/// 从借用返回迭代器
pub trait BorrowIntoIterator {
    type Item;

    fn iter(&self) -> impl Iterator<Item = Self::Item>;
}

/// 代表一个抽象的文件，提供一些文件的基本接口
pub trait AbstractFile : Clone {
    /// 获取父文件
    fn parent(&self) -> Option<Self>;

    /// 获取文件名
    fn name(&self) -> impl Deref<Target = String>;
    
    /// 获取哈希值
    fn hash(&self) -> impl Deref<Target = String>;
    
    /// 获取文件长度
    fn len(&self) -> u64;

    /// 获取文件修改时间
    fn modified(&self) -> SystemTime;
    
    /// 是不是一个目录
    fn is_dir(&self) -> bool;
    
    /// 获取文件的相对路径
    fn path(&self) -> impl Deref<Target = String>;
    
    /// 获取子文件列表
    fn files(&self) -> impl BorrowIntoIterator<Item = Self>;

    /// 搜索一个子文件，支持多层级搜索
    fn find(&self, path: &str) -> Option<Self>;
}

/// 查找文件的辅助函数，实现了大部分查找逻辑，可以很方便地直接使用
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

/// 计算相对路径的辅助函数，实现了大部分计算路径的逻辑，可以很方便地直接使用。
/// 
/// 但顶层目录的文件名不会被计算到结果中
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

/// 将抽象文件转换为调试字符串的辅助函数，可以输出很多有用的调试信息
pub fn abstract_file_to_string(f: &impl AbstractFile) -> String {
    if f.is_dir() {
        format!("{} (directory: {}) {}", &f.name().deref(), f.files().iter().count(), f.path().deref())
    } else {
        let dt = chrono::DateTime::<chrono::Local>::from(f.modified().to_owned());

        format!("{} ({}, {}, {}) {}", &f.name().deref(), f.len(), f.hash().deref(), dt.format("%Y-%m-%d %H:%M:%S"), f.path().deref())
    }
}

/// 遍历并输出所有层级下所有文件和目录的实用函数，主要用作调试用途
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