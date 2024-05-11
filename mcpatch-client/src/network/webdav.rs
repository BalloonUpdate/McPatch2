use std::future::Future;
use std::ops::Range;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use reqwest_dav::re_exports::reqwest::header::HeaderMap;
use reqwest_dav::re_exports::reqwest::header::HeaderName;
use reqwest_dav::re_exports::reqwest::Response;
use reqwest_dav::Client;
use reqwest_dav::ClientBuilder;
use tokio::io::AsyncRead;
use tokio::pin;

use crate::global_config::GlobalConfig;
use crate::network::DownloadResult;
use crate::network::UpdatingSource;

pub struct Webdav {
    client: Client
}

impl Webdav {
    pub fn new(host: String, user: String, pass: String, config: &GlobalConfig) -> Self {
        // 添加自定义协议头
        let mut def_headers = HeaderMap::new();

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

        Self { client }
    }
}

#[async_trait]
impl UpdatingSource for Webdav {
    async fn request<'a>(&'a mut self, path: &str, range: &Range<u64>, config: &GlobalConfig) -> DownloadResult<'a> {
        let rsp = match self.client.get(path).await {
            Ok(rsp) => rsp,
            Err(err) => return Err(err.to_string().into()),
        };

        let len = match rsp.content_length() {
            Some(len) => len,
            None => return Err(format!("the server doest not respond the content-length on {}", path).into()),
        };

        if len != range.end - range.start {
            return Err(format!("the content-length does not equal to {} on {}", range.end - range.start, path).into());
        }
        
        Ok((len, Box::pin(AsyncStreamBody(rsp))))
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