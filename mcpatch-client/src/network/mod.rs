pub mod http;
pub mod private;
pub mod webdav;

use std::ops::Range;
use std::pin::Pin;

use async_trait::async_trait;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

use crate::error::BusinessError;
use crate::error::BusinessResult;
use crate::global_config::GlobalConfig;
use crate::log::log_debug;
use crate::network::http::HttpProtocol;
use crate::network::private::PrivateProtocol;

pub type DownloadResult<'a> = BusinessResult<(u64, Pin<Box<dyn AsyncRead + Send + 'a>>)>;

pub struct Network<'a> {
    sources: Vec<Box<dyn UpdatingSource + Sync + Send>>,
    using_source: usize,
    config: &'a GlobalConfig,
}

impl<'a> Network<'a> {
    pub fn new(config: &'a GlobalConfig) -> Self {
        let mut sources = Vec::<Box<dyn UpdatingSource + Sync + Send>>::new();

        for url in &config.urls {
            if url.starts_with("http://") {
                sources.push(Box::new(HttpProtocol::new(url, &config)))
            } else if url.starts_with("mcpatch://") {
                sources.push(Box::new(PrivateProtocol::new(&url["mcpatch://".len()..], &config)))
            }
        }

        sources.push(Box::new(PrivateProtocol::new("127.0.0.1:6700", &config)));

        Network { sources, using_source: 0, config }
    }

    pub async fn request_text(&mut self, path: &str, range: Range<u64>, desc: impl AsRef<str>) -> BusinessResult<String> {
        let (len, mut data) = self.request_file(path, range, desc).await?;
        let mut text = String::with_capacity(len as usize);
        data.read_to_string(&mut text).await.unwrap();
        Ok(text)
    }

    pub async fn request_file<'b>(&'b mut self, path: &str, range: Range<u64>, desc: impl AsRef<str>) -> DownloadResult<'b> {
        let mut error = Option::<BusinessError>::None;
        let mut index = 0;
        
        for source in &mut self.sources {
            if index < self.using_source {
                index += 1;
                continue;
            }

            log_debug(format!("+ request {} {}+{} ({})", path, range.start, range.end - range.start, desc.as_ref()));

            assert!(range.end >= range.start);

            match source.request(path, &range, self.config).await {
                Ok(result) => return Ok(result),
                Err(err) => error = Some(err),
            }

            self.using_source += 1;
        }
        
        return Err(error.unwrap());
    }
}


#[async_trait]
pub trait UpdatingSource {
    async fn request<'a>(&'a mut self, path: &str, range: &Range<u64>, config: &GlobalConfig) -> DownloadResult<'a>;
}
