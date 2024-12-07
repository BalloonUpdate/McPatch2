use core::str;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;

use minio::s3::args::ListObjectsArgs;
use minio::s3::args::ObjectConditionalReadArgs;
use minio::s3::args::PutObjectArgs;
use minio::s3::args::RemoveObjectArgs;
use minio::s3::args::UploadObjectArgs;
use minio::s3::client::Client;
use minio::s3::client::ClientBuilder;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;

use crate::config::s3_config::S3Config;
use crate::upload::file_list_cache::FileListCache;
use crate::upload::SyncTarget;
use crate::utility::to_detail_error::ToDetailError;

pub struct S3Target {
    config: S3Config,
    client: Client,
}

impl S3Target {
    pub async fn new_cached(config: S3Config) -> FileListCache<Self> {
        FileListCache::new(Self::new(config).await)
    }

    pub async fn new(config: S3Config) -> Self {
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

        Self {
            config,
            client,
        }
    }
}

impl SyncTarget for S3Target {
    async fn list(&mut self) -> Result<Vec<String>, String> {

        let files = RefCell::new(Vec::<String>::new());
        
        self.client.list_objects(&ListObjectsArgs::new(&self.config.bucket, &|e| {
            let mut files = files.borrow_mut();

            for f in e {
                files.push(f.name);
            }

            true
        }).unwrap()).await.map_err(|e| e.to_detail_error())?;

        Ok(files.into_inner())
    }
    
    async fn read(&mut self, filename: &str) -> Result<Option<String>, String> {
        let response = self.client.get_object(&ObjectConditionalReadArgs::new(
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

        self.client.put_object(&mut PutObjectArgs::new(
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
        self.client.remove_object(&RemoveObjectArgs::new(
            &self.config.bucket,
            filename
        ).unwrap()).await.map_err(|e| e.to_detail_error())?;

        Ok(())
    }
}
