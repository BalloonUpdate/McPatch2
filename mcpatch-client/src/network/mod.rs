pub mod http;
pub mod private;

use std::ops::Range;
use std::pin::Pin;

use async_trait::async_trait;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

use crate::error::BusinessError;
use crate::error::BusinessResult;
use crate::global_config::GlobalConfig;
use crate::network::http::HttpProtocol;
use crate::network::private::PrivateProtocol;

pub type DownloadResult<'a> = BusinessResult<(u64, Pin<Box<dyn AsyncRead + 'a>>)>;

pub struct Network {
    sources: Vec<Box<dyn UpdatingSource + Sync>>
}

impl Network {
    pub fn new(config: &GlobalConfig) -> Self {
        let mut sources = Vec::<Box<dyn UpdatingSource + Sync>>::new();

        for url in &config.urls {
            if url.starts_with("http") {
                sources.push(Box::new(HttpProtocol::new(url)))
            } else if url.starts_with("mcpatch") {
                sources.push(Box::new(PrivateProtocol::new(url)))
            }
        }

        sources.push(Box::new(PrivateProtocol::new("127.0.0.1:6700")));

        Network { sources }
    }

    pub async fn request_text(&mut self, path: &str, range: Range<u64>) -> BusinessResult<String> {
        let (len, mut data) = self.request_file(path, range).await?;
        let mut text = String::with_capacity(len as usize);
        data.read_to_string(&mut text).await.unwrap();
        Ok(text)
    }

    pub async fn request_file<'a>(&'a mut self, path: &str, range: Range<u64>) -> DownloadResult<'a> {
        let mut error = Option::<BusinessError>::None;

        for source in &mut self.sources {
            match source.download(path, &range).await {
                Ok(result) => return Ok(result),
                Err(err) => error = Some(err),
            }
        }
        
        return Err(error.unwrap());
    }
}


#[async_trait]
pub trait UpdatingSource {
    async fn download<'a>(&'a mut self, path: &str, range: &Range<u64>) -> DownloadResult<'a>;
}
