pub mod s3;
pub mod webdav;
pub mod file_list_cache;

use std::future::Future;
use std::path::PathBuf;

pub trait UploadTarget {
    fn list(&mut self) -> impl Future<Output = Result<Vec<String>, String>>;

    fn read(&mut self, filename: &str) -> impl Future<Output = Result<Option<String>, String>>;

    fn write(&mut self, filename: &str, content: &str) -> impl Future<Output = Result<(), String>>;

    fn upload(&mut self, filename: &str, filepath: PathBuf) -> impl Future<Output = Result<(), String>>;

    fn delete(&mut self, filename: &str) -> impl Future<Output = Result<(), String>>;
}