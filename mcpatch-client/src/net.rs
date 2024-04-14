use std::io::ErrorKind;
use std::ops::Range;
use std::pin::Pin;

use async_trait::async_trait;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::global_config::GlobalConfig;
use crate::utility::limited_read_async::LimitedReadAsync;

pub struct Network {
    sources: Vec<Box<dyn UpdatingSource + Sync>>
}

impl Network {
    pub fn new(config: &GlobalConfig) -> Self {
        let sources = Vec::<Box<dyn UpdatingSource + Sync>>::new();

        for url in &config.urls {
            
        }

        Network { sources }
    }

    pub async fn request_text(&self, path: &str, range: Range<u64>) -> std::io::Result<String> {
        let (len, mut data) = self.request_file(path, range).await?;
        let mut text = String::with_capacity(len as usize);
        data.read_to_string(&mut text).await?;
        Ok(text)
    }

    pub async fn request_file<'a>(&'a self, path: &str, range: Range<u64>) -> std::io::Result<(u64, Pin<Box<dyn AsyncRead + 'a>>)> {
        for source in &self.sources {
            
        }

        todo!()
    }
}

#[async_trait]
pub trait UpdatingSource {
    async fn download<'a>(&'a mut self, path: &str) -> std::io::Result<(u64, Pin<Box<dyn AsyncRead + 'a>>)>;
}

pub struct PrivateProtocol {
    pub addr: String,
    pub tcp_stream: Option<TcpStream>,
}

impl PrivateProtocol {
    pub fn new(addr: String) -> Self {
        Self { 
            addr,
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
        let re = LimitedReadAsync::new(stream, len);

        Ok(re)
    }
}

#[async_trait]
impl UpdatingSource for PrivateProtocol {
    async fn download<'a>(&'a mut self, path: &str) -> std::io::Result<(u64, Pin<Box<dyn AsyncRead + 'a>>)> {
        self.send_data(path.as_bytes()).await?;

        let mut receive_code = self.receive_data().await?;
        let code = receive_code.read_u8().await?;

        if code == 0 {
            return Err(std::io::Error::new(ErrorKind::NotFound, "not found"));
        }

        let data = self.receive_data().await?;

        Ok((data.len(), Box::pin(data)))
    }
}

