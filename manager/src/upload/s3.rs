use tokio::io::AsyncRead;

use crate::upload::SyncTarget;

pub struct S3Target {

}

impl S3Target {
    pub async fn new() -> Self {
        Self { }
    }
}

// impl SyncTarget for S3Target {
//     async fn upload(path: &str, data: impl AsyncRead) {
//         todo!()
//     }

//     async fn download(path: &str) -> impl AsyncRead {
//         todo!()
//     }

//     async fn delete(path: &str) {
//         todo!()
//     }
// }