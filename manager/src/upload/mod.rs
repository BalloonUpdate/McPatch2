pub mod s3;

use std::future::Future;

use tokio::io::AsyncRead;

pub trait SyncTarget {
    fn upload(path: &str, data: impl AsyncRead) -> impl Future;

    fn download(path: &str) -> impl Future<Output = impl AsyncRead>;

    fn delete(path: &str) -> impl Future;
}