use core::str;
use std::collections::VecDeque;
use std::path::PathBuf;

use minio::s3::args::ObjectConditionalReadArgs;
use minio::s3::args::PutObjectArgs;
use minio::s3::args::UploadObjectArgs;
use minio::s3::client::Client;
use minio::s3::client::ClientBuilder;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::types::S3Api;
use minio::s3::types::ToStream;
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
        let response = self.client.get_object_old(&ObjectConditionalReadArgs::new(
            &self.config.bucket,
            filename
        ).unwrap()).await;

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

        let text = response.text().await.map_err(|e| e.to_detail_error())?;

        Ok(Some(text))
    }
    
    async fn write(&mut self, filename: &str, content: &str) -> Result<(), String> {
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
        self.client.upload_object(&UploadObjectArgs::new(
            &self.config.bucket,
            filename,
            filepath.canonicalize().unwrap().to_str().unwrap(),
        ).unwrap()).await.map_err(|e| e.to_detail_error())?;

        Ok(())
    }
    
    async fn delete(&mut self, filename: &str) -> Result<(), String> {
        let rsp = self.client.remove_object(&self.config.bucket, filename)
            .send()
            .await;

        rsp.map_err(|e| e.to_detail_error())?;

        Ok(())
    }
}
