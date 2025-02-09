use std::future::Future;
use std::ops::Range;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Url;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Response;
use tokio::io::AsyncRead;
use tokio::pin;
use serde_json::json;

use crate::error::BusinessError;
use crate::global_config::GlobalConfig;
// use crate::log::log_debug;
use crate::network::DownloadResult;
use crate::network::UpdatingSource;
use std::collections::HashMap;
use std::sync::Mutex;

/// 代表Alist更新协议
pub struct AlistProtocol {
    /// Alist更新地址的url
    pub url: String,

    /// http客户端对象
    pub client: Client,

    /// 打码的关键字，所有日志里的这个关键字都会被打码。通常用来保护服务器ip或者域名地址不被看到
    mask_keyword: String,

    /// 当前这个更新协议的编号，用来做debug用途
    index: u32,

    /// 缓存：存储 path -> raw_url
    cache: Mutex<HashMap<String, String>>,
}

impl AlistProtocol {
    pub fn new(url: &str, config: &GlobalConfig, index: u32) -> Self {
        // 确保 URL 末尾有 `/`
        let url = if url.ends_with('/') {
            url.to_owned()
        } else {
            format!("{}/", url)
        };

        // 添加自定义协议头
        let mut def_headers = HeaderMap::new();

        def_headers.insert("Content-Type", "application/json".parse().unwrap());

        for header in &config.http_headers {
            let k = HeaderName::from_str(&header.0).unwrap();
            let v = header.1.to_owned().parse().unwrap();
            def_headers.insert(k, v);
        }

        let client = ClientBuilder::new()
            .default_headers(def_headers)
            .connect_timeout(Duration::from_millis(config.http_timeout as u64))
            .read_timeout(Duration::from_millis(config.http_timeout as u64))
            .danger_accept_invalid_certs(config.http_ignore_certificate)
            .use_rustls_tls() // https://github.com/seanmonstar/reqwest/issues/2004#issuecomment-2180557375
            .build()
            .unwrap();

        let mask_keyword = match reqwest::Url::parse(&url) {
            Ok(parsed) => parsed.host_str().unwrap_or("").to_owned(),
            Err(_) => "".to_owned(),
        };

        Self { url: url.to_owned(), client, mask_keyword, index, cache: Mutex::new(HashMap::new()) }
    }
}

#[async_trait]
impl UpdatingSource for AlistProtocol {
    async fn request(&mut self, path: &str, range: &Range<u64>, desc: &str, _config: &GlobalConfig) -> DownloadResult {
        let cached_url = {
            let cache = self.cache.lock().unwrap();
            cache.get(path).cloned()
        };

        if let Some(raw_url) = cached_url {
            // log_debug(format!("Using cached raw_url for path: {}", path));
            return self.fetch_file_with_url(&raw_url, range, path, desc).await;
        } else {
            // 获取实际的文件URL
            let base_url: Url = match Url::parse(&self.url) {
                Ok(url) => url,
                Err(_) => return Ok(Err(BusinessError::new(format!("无效的基础URL: {}", self.url)))),
            };

            // 解析基础URL失败
            let host_url = match base_url.join("/") {
                Ok(url) => url,
                Err(_) => return Ok(Err(BusinessError::new(format!("解析基础URL失败: {}", self.url)))),
            };

            // 拼接 "/api/fs/get" 和 path
            let full_url = match base_url.join("/api/fs/get") {
                Ok(url) => url,
                Err(_) => return Ok(Err(BusinessError::new(format!("拼接路径失败: {} 和 /api/fs/get", self.url)))),
            };

            let real_path = format!("/{}", (base_url.as_str().replace(host_url.as_str(), "") + path));

            // 请求负载
            let payload = json!({
                "path": real_path,
                "password": "",
            });

            // log_debug(format!("alist base_url {}", base_url));
            // log_debug(format!("alist host_url {}", host_url));
            // log_debug(format!("alist full_url {}", full_url));
            // log_debug(format!("alist real_path {}", real_path));


            // 发起POST请求
            let rsp = match self.client.post(full_url)
                .json(&payload)
                .send()
                .await
            {
                Ok(rsp) => rsp,
                Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
            };

            // 输出服务器返回的原始响应内容
            let response_text = match rsp.text().await {
                Ok(text) => text,
                Err(_) => return Ok(Err(BusinessError::new(format!("服务器({})返回的内容无法读取: {} ({})", self.index, path, desc)))),
            };

            // log_debug(format!("服务器({})返回的原始内容: {}", self.index, response_text));

            // 解析JSON响应
            let json: serde_json::Value = match serde_json::from_str(&response_text) {
                Ok(json) => json,
                Err(_) => return Ok(Err(BusinessError::new(format!("服务器({})返回的数据格式不正确: {} ({})", self.index, path, desc)))),
            };

            // 提取实际URL
            let raw_url = json.pointer("/data/raw_url").and_then(serde_json::Value::as_str);

            if let Some(raw_url) = raw_url {
                {
                    let mut cache = self.cache.lock().unwrap();
                    cache.insert(path.to_string(), raw_url.to_string());
                }
                return self.fetch_file_with_url(&raw_url, range, path, desc).await;
            } else {
                return Ok(Err(BusinessError::new(format!("服务器({})返回的数据中没有raw_url字段: {} ({})", self.index, path, desc))));
            }

        }
    }

    fn mask_keyword(&self) -> &str {
        &self.mask_keyword
    }
}

impl AlistProtocol {
    async fn fetch_file_with_url(
        &self,
        url: &str,
        range: &Range<u64>,
        path: &str,
        desc: &str,
    ) -> DownloadResult {
        // log_debug(format!("服务器({})返回的实际URL: {}", self.index, url));

        // 检查输入参数，start不能大于end
        let partial_file = !(range.start == 0 && range.end == 0) && (range.start > 0 || range.end > 0);

        if partial_file {
            assert!(range.end >= range.start);
        }

        // 构建请求
        let mut req = self.client.get(url);

        // 如果是分段请求，添加Range头
        if partial_file {
            req = req.header("Range", format!("bytes={}-{}", range.start, range.end - 1));
        }

        let req = req.build().unwrap();

        // 发起请求
        let rsp = match self.client.execute(req).await {
            Ok(rsp) => rsp,
            Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
        };

        let code = rsp.status().as_u16();

        // 检查状态码
        if (!partial_file && (code < 200 || code >= 300)) || (partial_file && code != 206) {
            let mut body = rsp.text().await.map_or_else(|e| format!("{:?}", e), |v| v);
            // log_debug(format!("------------\n{}\n------------", body));
            body.truncate(300);

            return Ok(Err(BusinessError::new(format!("服务器({})返回了{}而不是206: {} ({})", self.index, code, path, desc))));
        }

        // 如果是分段请求，检查content-length
        if partial_file {
            let len = match rsp.content_length() {
                Some(len) => len,
                None => return Ok(Err(BusinessError::new(format!("服务器({})没有返回content-length头: {} ({})", self.index, path, desc)))),
            };

            if (range.end - range.start) > 0 && len != range.end - range.start {
                return Ok(Err(BusinessError::new(format!("服务器({})返回的content-length头 {} 不等于{}: {}", self.index, len, range.end - range.start, path))));
            }
        }

        Ok(Ok((rsp.content_length().unwrap_or(0), Box::pin(AsyncStreamBody(rsp, None)))))
    }
}

pub struct AsyncStreamBody(pub Response, pub Option<bytes::Bytes>);

impl AsyncRead for AsyncStreamBody {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if buf.remaining() == 0 {
            return std::task::Poll::Ready(Ok(()));
        }

        if self.1.is_none() {
            let bytes = {
                let chunk = self.0.chunk();
                pin!(chunk);
        
                match chunk.poll(cx) {
                    std::task::Poll::Ready(Ok(Some(chunk))) => chunk,
                    std::task::Poll::Ready(Ok(None)) => return std::task::Poll::Ready(Ok(())),
                    std::task::Poll::Ready(Err(err)) => return std::task::Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, err))),
                    std::task::Poll::Pending => return std::task::Poll::Pending,
                }
            };

            self.1 = Some(bytes);
        }

        let holding = self.1.as_mut().unwrap();
        let count = buf.remaining().min(holding.len());

        buf.put_slice(&holding.split_to(count));

        if holding.len() == 0 {
            self.1 = None;
        }

        std::task::Poll::Ready(Ok(()))
    }
}
