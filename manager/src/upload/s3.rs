use core::str;
use std::collections::VecDeque;
use std::path::PathBuf;

use axum::body::Bytes;
use minio::s3::args::PutObjectArgs;
use minio::s3::client::Client;
use minio::s3::client::ClientBuilder;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::types::PartInfo;
use minio::s3::types::S3Api;
use minio::s3::types::ToStream;
use tokio::io::AsyncReadExt;
use tokio_stream::StreamExt;

use crate::config::s3_config::S3Config;
use crate::upload::file_list_cache::FileListCache;
use crate::upload::UploadTarget;
use crate::utility::to_detail_error::ToDetailError;

pub struct S3Target {
    config: S3Config,
    client: Client,
}

impl S3Target {
    pub async fn new(config: S3Config) -> FileListCache<Self> {
        let base_url = config.endpoint.parse::<BaseUrl>().unwrap();

        let static_provider = StaticProvider::new(
            &config.access_id,
            &config.secret_key,
            None,
        );

        let client = ClientBuilder::new(base_url)
            .provider(Some(Box::new(static_provider)))
            .build()
            .unwrap();

        FileListCache::new(Self {
            config,
            client,
        })
    }
}

impl UploadTarget for S3Target {
    async fn list(&mut self) -> Result<Vec<String>, String> {
        let mut files = Vec::<String>::new();

        let mut list_objects = self.client
            .list_objects(&self.config.bucket)
            .recursive(false)
            .to_stream()
            .await;

        while let Some(result) = list_objects.next().await {
            let rsp = result.map_err(|e| e.to_detail_error())?;

            for item in rsp.contents {
                files.push(item.name);
            }
        }

        Ok(files)
    }
    
    async fn read(&mut self, filename: &str) -> Result<Option<String>, String> {
        println!("read");
        let response = self.client.get_object(&self.config.bucket, filename)
            .send()
            .await;
        // let response = self.client.get_object_old(&ObjectConditionalReadArgs::new(
        //     &self.config.bucket,
        //     filename
        // ).unwrap()).await;

        let response = match response {
            Ok(ok) => ok,
            Err(err) => {
                if let minio::s3::error::Error::S3Error(err_rsp) = &err {
                    if err_rsp.code == "NoSuchKey" {
                        return Ok(None);
                    }
                }

                return Err(err.to_detail_error());
            },
        };

        let text = response.content.to_segmented_bytes()
            .await.map(|e| String::from_utf8(e.to_bytes().into_iter().collect::<Vec<u8>>()))
            .map_err(|e| e.to_detail_error())?
            .map_err(|e| e.to_detail_error())?;

        Ok(Some(text))
    }
    
    async fn write(&mut self, filename: &str, content: &str) -> Result<(), String> {
        println!("write");
        let mut buf = VecDeque::from(content.as_bytes().to_vec());
        let len = buf.len();

        self.client.put_object_old(&mut PutObjectArgs::new(
            &self.config.bucket,
            filename,
            &mut buf,
            Some(len),
            None,
        ).unwrap()).await.map_err(|e| e.to_detail_error())?;

        Ok(())
    }
    
    async fn upload(&mut self, filename: &str, filepath: PathBuf) -> Result<(), String> {
        println!("upload");

        let filepath = filepath.canonicalize().unwrap().to_str().unwrap().to_owned();
        let len = tokio::fs::metadata(&filepath).await.unwrap().len();

        let create_multipart = self.client.create_multipart_upload(&self.config.bucket, filename).send().await
            .map_err(|e| e.to_detail_error())?;

        let mut file = tokio::fs::OpenOptions::new()
            .read(true)
            .open(&filepath)
            .await
            .unwrap();

        let upload_id = create_multipart.upload_id;
        let mut parts = Vec::<PartInfo>::new();
        
        let part_size = 16 * 1024 * 1024;
        let part_count = (len / part_size) as usize;
        let part_remains = (len % part_size) as usize;

        let mut buf = Vec::<u8>::with_capacity(part_size as usize);
        buf.resize(part_size as usize, 0);

        let total_count = part_count + (if part_remains > 0 { 1 } else { 0 });
        for i in 0..total_count {
            let part = (i + 1) as u16;
            let read = match i == total_count - 1 {
                true => file.read_exact(&mut buf[0..part_remains]).await.unwrap(),
                false => file.read_exact(&mut buf).await.unwrap(),
            };
            let data = &buf[0..read];

            let bytes = Bytes::copy_from_slice(&data);

            let rsp = self.client.upload_part(&self.config.bucket, filename, &upload_id, part, bytes.into()).send().await
                .map_err(|e| e.to_detail_error())?;

            parts.push(PartInfo {
                number: part,
                etag: rsp.etag,
                size: read as u64,
            });
        }

        self.client.complete_multipart_upload(&self.config.bucket, filename, &upload_id, parts).send().await
            .map_err(|e| e.to_detail_error())?;

        Ok(())
    }
    
    async fn delete(&mut self, filename: &str) -> Result<(), String> {
        println!("delete");
        let rsp = self.client.remove_object(&self.config.bucket, filename)
            .send()
            .await;

        rsp.map_err(|e| e.to_detail_error())?;

        Ok(())
    }
}
