use std::ops::Range;

use async_trait::async_trait;
use mcpatch_shared::utility::limited_read_async::LimitedReadAsync;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::network::DownloadResult;
use crate::network::UpdatingSource;

pub struct PrivateProtocol {
    pub addr: String,
    pub tcp_stream: Option<TcpStream>,
}

impl PrivateProtocol {
    pub fn new(url: &str) -> Self {
        Self { 
            addr: url.to_owned(),
            tcp_stream: None,
        }
    }

    async fn get_tcp_stream(&mut self) -> std::io::Result<&mut TcpStream> {
        if self.tcp_stream.is_none() {
            self.tcp_stream = Some(TcpStream::connect(&self.addr).await?);
        }

        Ok(self.tcp_stream.as_mut().unwrap())
    }

    async fn send_data(&mut self, data: &[u8]) -> std::io::Result<()> {
        let stream = self.get_tcp_stream().await?;

        stream.write_u64_le(data.len() as u64).await?;
        stream.write_all(data).await?;

        Ok(())
    }

    async fn receive_data(&mut self) -> std::io::Result<LimitedReadAsync<'_, TcpStream>> {
        let stream = self.get_tcp_stream().await?;

        let len = stream.read_u64_le().await?;

        Ok(LimitedReadAsync::new(stream, len))
    }
}

#[async_trait]
impl UpdatingSource for PrivateProtocol {
    async fn download<'a>(&'a mut self, path: &str, range: Range<u64>) -> DownloadResult<'a> {
        // 首先发送文件路径
        self.send_data(path.as_bytes()).await.unwrap();
        
        // 然后发送下载范围
        self.send_data(&range.start.to_le_bytes()).await.unwrap();
        self.send_data(&range.end.to_le_bytes()).await.unwrap();

        // 接收状态码
        let code = self.receive_data().await.unwrap().read_u8().await.unwrap();

        if code != 0 {
            return Err(format!("error {} on receiving {}", code, path).into());
        }

        // 状态码没问题就正常接收文件数据
        let data = self.receive_data().await.unwrap();

        Ok((data.len(), Box::pin(data)))
    }
}
