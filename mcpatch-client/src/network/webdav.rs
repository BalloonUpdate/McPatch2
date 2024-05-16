use std::future::Future;
use std::ops::Range;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use reqwest_dav::re_exports::reqwest::header::HeaderMap;
use reqwest_dav::re_exports::reqwest::header::HeaderName;
use reqwest_dav::re_exports::reqwest::Method;
use reqwest_dav::re_exports::reqwest::Response;
use reqwest_dav::Client;
use reqwest_dav::ClientBuilder;
use tokio::io::AsyncRead;
use tokio::pin;

use crate::error::BusinessError;
use crate::global_config::GlobalConfig;
use crate::network::DownloadResult;
use crate::network::UpdatingSource;

pub struct Webdav {
    client: Client,
    index: u32,
}

impl Webdav {
    pub fn new(url: &str, config: &GlobalConfig, index: u32) -> Self {
        let split_at = url.find("://").unwrap() + 3;
        let scheme = &url[0..split_at].replace("webdav", "http");
        let str = &url[split_at..];
        let parsed = str.splitn(3, ":").collect::<Vec<_>>();
        let user = parsed[0].to_owned();
        let pass = parsed[1].to_owned();
        let host = format!("{}{}", scheme, parsed[2]);
        
        // 添加自定义协议头
        let mut def_headers = HeaderMap::new();

        println!("host: {}, user: {}, pass: {}", host, user, pass);

        for header in &config.http_headers {
            let k = HeaderName::from_str(&header.0).unwrap();
            let v = header.1.to_owned().parse().unwrap();
            def_headers.insert(k, v);
        }
        
        let reqwest_client = reqwest_dav::re_exports::reqwest::ClientBuilder::new()
            .default_headers(def_headers)
            .timeout(Duration::from_millis(config.http_timeout as u64))
            .danger_accept_invalid_certs(config.http_ignore_certificate)
            .build().unwrap();

        let client = ClientBuilder::new()
            .set_agent(reqwest_client)
            .set_host(host)
            .set_auth(reqwest_dav::Auth::Basic(user, pass))
            .build()
            .unwrap();

        Self { client, index }
    }
}

#[async_trait]
impl UpdatingSource for Webdav {
    async fn request(&mut self, path: &str, range: &Range<u64>, desc: &str, _config: &GlobalConfig) -> DownloadResult {
        let partial_file = range.start > 0 || range.end > 0;

        if partial_file {
            assert!(range.end >= range.start);
        }

        let req = match self.client.start_request(Method::GET, &path).await {
            Ok(mut builder) => {
                if partial_file {
                    builder = builder.header("Range", format!("bytes={}-{}", range.start, range.end - 1));
                }
                builder
            },
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        };

        let rsp = match req.send().await {
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

        if partial_file && len != range.end - range.start {
            return Ok(Err(BusinessError::new(format!("服务器({})返回的content-length头 {} 不等于{}: {} ({})", self.index, len, range.end - range.start, path, desc))));
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