use std::future::Future;
use std::ops::Range;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Response;
use tokio::io::AsyncRead;
use tokio::pin;

use crate::error::BusinessError;
use crate::global_config::GlobalConfig;
use crate::network::DownloadResult;
use crate::network::UpdatingSource;

pub struct HttpProtocol {
    pub url: String,
    pub client: Client,
    pub range_bytes_supported: bool,
    index: u32,
}

impl HttpProtocol {
    pub fn new(url: &str, config: &GlobalConfig, index: u32) -> Self {
        // 添加自定义协议头
        let mut def_headers = HeaderMap::new();

        def_headers.insert("Content-Type", "application/octet-stream".parse().unwrap());

        for header in &config.http_headers {
            let k = HeaderName::from_str(&header.0).unwrap();
            let v = header.1.to_owned().parse().unwrap();
            def_headers.insert(k, v);
        }

        let client = ClientBuilder::new()
            .default_headers(def_headers)
            .timeout(Duration::from_millis(config.http_timeout as u64))
            .danger_accept_invalid_certs(config.http_ignore_certificate)
            .build()
            .unwrap();

        Self { url: url.to_owned(), client, range_bytes_supported: false, index }
    }
}

#[async_trait]
impl UpdatingSource for HttpProtocol {
    async fn request(&mut self, path: &str, range: &Range<u64>, desc: &str, _config: &GlobalConfig) -> DownloadResult {
        let full_url = format!("{}{}{}", self.url, if self.url.ends_with("/") { "" } else { "/" }, path);

        let partial_file = range.start > 0 || range.end > 0;

        if partial_file {
            assert!(range.end >= range.start);
        }

        let mut req = self.client.get(&full_url);
        if partial_file {
            req = req.header("Range", format!("bytes={}-{}", range.start, range.end - 1));
        }
        let req = req.build().unwrap();

        let rsp = match self.client.execute(req).await {
            Ok(rsp) => rsp,
            Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
        };

        let code = rsp.status().as_u16();

        if partial_file && code != 206 {
            return Ok(Err(BusinessError::new(format!("服务器({})返回了{}而不是206: {} ({})", self.index, code, path, desc))));
        }

        let len = match rsp.content_length() {
            Some(len) => len,
            None => return Ok(Err(BusinessError::new(format!("服务器({})没有返回content-length头: {} ({})", self.index, path, desc)))),
        };

        if (range.end - range.start) > 0 && len != range.end - range.start {
            return Ok(Err(BusinessError::new(format!("服务器({})返回的content-length头 {} 不等于{}: {}", self.index, len, range.end - range.start, path))));
        }
        
        Ok(Ok((len, Box::pin(AsyncStreamBody(rsp)))))
    }
}

pub struct AsyncStreamBody(pub Response);

impl AsyncRead for AsyncStreamBody {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let chunk = self.0.chunk();
        pin!(chunk);

        let bytes = match chunk.poll(cx) {
            std::task::Poll::Ready(r) => match r {
                Ok(bytes) => bytes,
                Err(e) => {
                    let err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, e);

                    return std::task::Poll::Ready(Err(err))
                },
            },
            std::task::Poll::Pending => return std::task::Poll::Pending,
        };

        if let Some(bytes) = bytes {
            buf.put_slice(&bytes);
        }

        std::task::Poll::Ready(Ok(()))
    }
}