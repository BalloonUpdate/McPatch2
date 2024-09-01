use std::path::Path;
use std::path::PathBuf;

/// 给PathBuf和Path增加快速获取文件名的扩展方法
pub trait GetFileNamePart {
    /// 返回文件名部分
    fn filename(&self) -> &str;
}

impl GetFileNamePart for PathBuf {
    fn filename(&self) -> &str {
        self.file_name().unwrap().to_str().unwrap()
    }
}

impl GetFileNamePart for Path {
    fn filename(&self) -> &str {
        self.file_name().unwrap().to_str().unwrap()
    }
}