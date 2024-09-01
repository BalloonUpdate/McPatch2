use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::pin;
use tokio::sync::Mutex;
use tokio::sync::OwnedMutexGuard;

use crate::error::BusinessError;
use crate::global_config::GlobalConfig;
use crate::network::DownloadResult;
use crate::network::UpdatingSource;

pub struct PrivateProtocol {
    pub addr: String,
    pub tcp_stream: Arc<Mutex<Option<TcpStream>>>,
    mask_keyword: String,
    index: u32,
}

impl PrivateProtocol {
    pub fn new(addr: &str, _config: &GlobalConfig, index: u32) -> Self {
        Self { 
            addr: addr.to_owned(),
            tcp_stream: Arc::new(Mutex::new(None)),
            mask_keyword: addr.to_owned(),
            index,
        }
    }
}

#[async_trait]
impl UpdatingSource for PrivateProtocol {
    async fn request(&mut self, path: &str, range: &Range<u64>, desc: &str, config: &GlobalConfig) -> DownloadResult {
        let mut stream_lock = self.tcp_stream.clone().lock_owned().await;

        if stream_lock.is_none() {
            let tcp = TcpStream::connect(&self.addr).await?;
            let std_tcp = tcp.into_std().unwrap();
            std_tcp.set_read_timeout(Some(Duration::from_millis(config.private_timeout as u64))).unwrap();
            std_tcp.set_write_timeout(Some(Duration::from_millis(config.private_timeout as u64))).unwrap();

            *stream_lock = Some(tokio::net::TcpStream::from_std(std_tcp).unwrap());
        }

        let index = self.index;
        let stream = stream_lock.as_mut().unwrap();

        // 首先发送文件路径
        send_data(stream, path.as_bytes()).await?;
        
        // 然后发送下载范围
        stream.write_all(&range.start.to_le_bytes()).await?;
        stream.write_all(&range.end.to_le_bytes()).await?;

        // 接收状态码或者文件大小
        let len = stream.read_i64_le().await?;

        if len < 0 {
            return Ok(Err(BusinessError::new(format!("私有协议({})接收到的状态码 {} 不正确: {} ({})", index, len, path, desc))));
        }

        // 状态码没问题就正常接收文件数据
        let data = receive_partial_data(stream_lock, len as u64).await?;

        Ok(Ok((data.count(), Box::pin(data))))
    }

    fn mask_keyword(&self) -> &str {
        &self.mask_keyword
    }
}

async fn send_data(stream: &mut TcpStream, data: &[u8]) -> std::io::Result<()> {
    stream.write_u64_le(data.len() as u64).await?;
    stream.write_all(data).await?;

    Ok(())
}

async fn _receive_data<'a>(mut stream: OwnedMutexGuard<Option<TcpStream>>) -> std::io::Result<PrivatePartialAsyncRead> {
    let len = stream.as_mut().unwrap().read_u64_le().await?;

    Ok(PrivatePartialAsyncRead::new(stream, len))
}

async fn receive_partial_data<'a>(stream: OwnedMutexGuard<Option<TcpStream>>, count: u64) -> std::io::Result<PrivatePartialAsyncRead> {
    Ok(PrivatePartialAsyncRead::new(stream, count))
}

/// 代表一个限制读取数量的Read
pub struct PrivatePartialAsyncRead(OwnedMutexGuard<Option<TcpStream>>, u64);

impl PrivatePartialAsyncRead {
    pub fn new(tcp: OwnedMutexGuard<Option<TcpStream>>, count: u64) -> Self {
        Self(tcp, count)
    }

    pub fn count(&self) -> u64 {
        self.1
    }
}

impl AsyncRead for PrivatePartialAsyncRead {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.1 == 0 {
            return std::task::Poll::Ready(Ok(()));
        }

        let limit = buf.remaining().min(self.1 as usize);
        let partial = &mut buf.take(limit);

        let read = self.0.as_mut().unwrap();
        pin!(read);

        match read.poll_read(cx, partial) {
            std::task::Poll::Ready(ready) => {
                let adv = partial.filled().len();

                unsafe { buf.assume_init(adv); }
                buf.advance(adv);

                self.1 -= adv as u64;

                std::task::Poll::Ready(ready)
            },
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}