use std::path::Path;
use std::path::PathBuf;

pub trait GetFileNamePart {
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