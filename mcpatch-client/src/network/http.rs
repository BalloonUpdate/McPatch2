use std::future::Future;
use std::ops::Range;
use std::pin::Pin;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Response;
use tokio::io::AsyncRead;
use tokio::pin;

use crate::network::DownloadResult;
use crate::network::UpdatingSource;

pub struct HttpProtocol {
    pub url: String,
    pub client: Client,
    pub range_bytes_supported: bool,
}

impl HttpProtocol {
    pub fn new(url: &str) -> Self {
        let mut def_headers = HeaderMap::new();

        // def_headers.insert("Host", host.parse().unwrap());
        def_headers.insert("Content-Type", "application/octet-stream".parse().unwrap());

        let client = ClientBuilder::new()
            .default_headers(def_headers)
            .timeout(Duration::from_secs(60))
            .build().unwrap();

        Self { url: url.to_owned(), client, range_bytes_supported: false }
    }
}

#[async_trait]
impl UpdatingSource for HttpProtocol {
    async fn download<'a>(&'a mut self, path: &str, range: &Range<u64>) -> DownloadResult<'a> {
        assert!(range.end - range.start > 0);

        let full_url = format!("{}{}", self.url, path);

        let req = self.client.get(&full_url)
            .header("Range", format!("bytes={}-{}", range.start, range.end))
            .build().unwrap();

        let rsp = match self.client.execute(req).await {
            Ok(rsp) => rsp,
            Err(err) => return Err(err.to_string().into()),
        };

        let code = rsp.status().as_u16();

        if code != 206 {
            return Err(format!("the http code returned {} is not 206 on {}", code, full_url).into());
        }

        let len = match rsp.content_length() {
            Some(len) => len,
            None => return Err(format!("the server doest not respond the content-length on {}", full_url).into()),
        };

        if len != range.end - range.start {
            return Err(format!("the content-length does not equal to {} on {}", range.end - range.start, full_url).into());
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