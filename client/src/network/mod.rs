pub mod http;
pub mod private;
pub mod webdav;

use std::ops::Range;
use std::pin::Pin;

use async_trait::async_trait;
use shared::utility::is_running_under_cargo;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

use crate::error::BusinessError;
use crate::error::BusinessResult;
use crate::error::ResultToBusinessError;
use crate::global_config::GlobalConfig;
use crate::log::log_debug;
use crate::log::log_error;
use crate::log::log_info;
use crate::network::http::HttpProtocol;
use crate::network::private::PrivateProtocol;
use crate::network::webdav::Webdav;

pub type DownloadResult = std::io::Result<BusinessResult<(u64, Pin<Box<dyn AsyncRead + Send>>)>>;

pub struct Network<'a> {
    sources: Vec<Box<dyn UpdatingSource + Sync + Send>>,
    skip_sources: usize,
    config: &'a GlobalConfig,
}

impl<'a> Network<'a> {
    pub fn new(config: &'a GlobalConfig) -> BusinessResult<Self> {
        let mut sources = Vec::<Box<dyn UpdatingSource + Sync + Send>>::new();
        let mut index = 0u32;

        if is_running_under_cargo() {
            for (index, url) in config.urls.iter().enumerate() {
                log_debug(format!("{}: {}", index, url));
            }
        }

        for url in &config.urls {
            if url.starts_with("http://") || url.starts_with("https://") {
                sources.push(Box::new(HttpProtocol::new(url, &config, index)))
            } else if url.starts_with("mcpatch://") {
                sources.push(Box::new(PrivateProtocol::new(&url["mcpatch://".len()..], &config, index)))
            } else if url.starts_with("webdav://") || url.starts_with("webdavs://") {
                sources.push(Box::new(Webdav::new(&url, &config, index)))
            } else {
                log_info(format!("unknown url: {}", url));
            }

            index += 1;
        }

        log_debug(format!("loaded {} urls", sources.len()));

        if sources.len() == 0 {
            return Err(BusinessError::new("没有有效的服务器地址可以使用"));
        }

        Ok(Network { sources, skip_sources: 0, config })
    }

    pub async fn request_text(&mut self, path: &str, range: Range<u64>, desc: impl AsRef<str>) -> BusinessResult<String> {
        match self.request_file(path, range, desc.as_ref()).await {
            Ok(ok) => {
                let (len, mut data) = ok;
                
                let mut text = String::with_capacity(len as usize);
                data.read_to_string(&mut text).await.be(|e| format!("网络数据无法解码为utf8字符串({})，原因：{:?}", desc.as_ref(), e))?;
                Ok(text)
            },
            Err(err) => return Err(err),
        }
    }

    pub async fn request_file(&mut self, path: &str, range: Range<u64>, desc: &str) -> BusinessResult<(u64, Pin<Box<dyn AsyncRead + Send>>)> {
        assert!(range.end >= range.start);

        let mut io_error = Option::<(std::io::Error, String)>::None;
        
        for (index, source) in (&mut self.sources[self.skip_sources..]).iter_mut().enumerate() {
            let url_index = self.skip_sources + index;

            log_debug(format!("+ request {} {}+{} ({}) url: {}", path, range.start, range.end - range.start, desc, url_index));

            for i in 0..self.config.http_retries + 1 {
                match source.request(path, &range, desc.as_ref(), self.config).await {
                    Ok(ok) => {
                        match ok {
                            Ok(ok) => return Ok(ok),
                            Err(err) => return Err(err),
                        }
                    },
                    Err(err) => {
                        io_error = Some((err, source.mask_keyword().to_owned()));
                        
                        if i != self.config.http_retries {
                            log_error(format!("url {} failed, retrying...", url_index));
                        }
                    },
                }
            }
        }
        
        let (err, kw) = io_error.unwrap();
        return Err(BusinessError::new(format!("{:?}", err).replace(&kw, "[主机部分]")));
    }

    pub fn advance_source(&mut self) {
        self.skip_sources += 1;
    }
}

#[async_trait]
pub trait UpdatingSource {
    /// 发起一个文件请求
    async fn request(&mut self, path: &str, range: &Range<u64>, desc: &str, config: &GlobalConfig) -> DownloadResult;

    /// 返回主机部分的关键字，用来日志中的字符串打码，遮住其中的主机部分
    fn mask_keyword(&self) -> &str;
}
