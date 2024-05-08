use std::fmt::format;
use std::ops::Range;

use async_trait::async_trait;
use mcpatch_shared::utility::partial_read::PartialAsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::log::log_error;
use crate::network::DownloadResult;
use crate::network::UpdatingSource;

pub struct PrivateProtocol {
    pub addr: String,
    pub tcp_stream: Option<TcpStream>,
}

impl PrivateProtocol {
    pub fn new(addr: &str) -> Self {
        Self { 
            addr: addr.to_owned(),
            tcp_stream: None,
        }
    }

    async fn stream(&mut self) -> std::io::Result<&mut TcpStream> {
        if self.tcp_stream.is_none() {
            // log_error(format!("{}", self.addr));

            self.tcp_stream = Some(TcpStream::connect(&self.addr).await?);
        }

        Ok(self.tcp_stream.as_mut().unwrap())
    }

    async fn send_data(&mut self, data: &[u8]) -> std::io::Result<()> {
        let stream = self.stream().await?;

        stream.write_u64_le(data.len() as u64).await?;
        stream.write_all(data).await?;

        Ok(())
    }

    async fn _receive_data(&mut self) -> std::io::Result<PartialAsyncRead<'_, TcpStream>> {
        let stream = self.stream().await?;

        let len = stream.read_u64_le().await?;

        Ok(PartialAsyncRead::new(stream, len))
    }

    async fn receive_partial_data(&mut self, count: u64) -> std::io::Result<PartialAsyncRead<'_, TcpStream>> {
        let stream = self.stream().await?;

        Ok(PartialAsyncRead::new(stream, count))
    }
}

#[async_trait]
impl UpdatingSource for PrivateProtocol {
    async fn request<'a>(&'a mut self, path: &str, range: &Range<u64>) -> DownloadResult<'a> {
        // 首先发送文件路径
        self.send_data(path.as_bytes()).await.unwrap();
        
        // 然后发送下载范围
        self.stream().await.unwrap().write_all(&range.start.to_le_bytes()).await.unwrap();
        self.stream().await.unwrap().write_all(&range.end.to_le_bytes()).await.unwrap();

        // 接收状态码或者文件大小
        let len = self.stream().await.unwrap().read_i64_le().await.unwrap();

        if len < 0 {
            return Err(format!("error code {} on receiving {}", len, path).into());
        }

        // 状态码没问题就正常接收文件数据
        let data = self.receive_partial_data(len as u64).await.unwrap();

        Ok((data.count(), Box::pin(data)))
    }
}
