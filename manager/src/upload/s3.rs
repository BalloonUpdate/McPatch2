use core::str;
use std::path::PathBuf;

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::CompletedMultipartUpload;
use aws_sdk_s3::types::CompletedPart;
use aws_sdk_s3::Client;
use tokio::io::AsyncReadExt;

use crate::config::s3_config::S3Config;
use crate::upload::file_list_cache::FileListCache;
use crate::upload::SyncTarget;

pub struct S3Target {
    config: S3Config,
    client: Client,
}

impl S3Target {
    pub async fn new_cached(config: S3Config) -> FileListCache<Self> {
        FileListCache::new(Self::new(config).await)
    }

    pub async fn new(config: S3Config) -> Self {
        // cos.ap-guangzhou.myqcloud.com
        let cfg = aws_sdk_s3::config::Builder::new()
            .endpoint_url(config.endpoint.clone())
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                config.access_id.clone(),
                config.secret_key.clone(),
                None,
                None,
                "mcpatch-provider"
            ))
            .build();
        
        let client = aws_sdk_s3::Client::from_conf(cfg);

        Self {
            config,
            client,
        }
    }
}

impl SyncTarget for S3Target {
    async fn list(&mut self) -> Result<Vec<String>, String> {
        let list_rsp = self.client
            .list_objects()
            .bucket(&self.config.bucket)
            // .key("")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(list_rsp.contents().iter().map(|e| e.key().unwrap().to_owned()).collect())
    }
    
    async fn read(&mut self, filename: &str) -> Result<String, String> {
        let result = self.client.get_object()
            .bucket(&self.config.bucket)
            .key(filename)
            .send()
            .await;

        let read = result.map_err(|e| e.to_string())?;

        let text = std::str::from_utf8(read.body.bytes().unwrap()).unwrap().to_owned();

        Ok(text)
    }
    
    async fn write(&mut self, filename: &str, content: &str) -> Result<(), String> {
        let _result = self.client.put_object()
            .bucket(&self.config.bucket)
            .key(filename)
            .body(ByteStream::from(content.as_bytes().to_vec()))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
    
    async fn upload(&mut self, filename: &str, filepath: PathBuf) -> Result<(), String> {
        let metadata = tokio::fs::metadata(&filepath).await.unwrap();
        let file_size = metadata.len();

        let file = tokio::fs::File::open(&filepath).await.unwrap();
        let mut file = tokio::io::BufReader::new(file);

        // 准备分块上传
        let create_multipart_resp = self.client
            .create_multipart_upload()
            .bucket(&self.config.bucket)
            .key(filename)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let upload_id = create_multipart_resp.upload_id.unwrap();

        let mut part_number = 1;

        let mut complete_parts = CompletedMultipartUpload::builder();
        let mut buffer = vec![0; 8 * 1024 * 1024];

        // 分块上传
        let mut uploaded = 0;
        
        while uploaded < file_size {
            let read_size = file.read(&mut buffer).await.unwrap();

            // 完成上传
            if read_size == 0 {
                break;
            }

            // 上传当前块
            let body = ByteStream::from(buffer[..read_size].to_vec());

            let rsp = self.client
                .upload_part()
                .bucket(&self.config.bucket)
                .key(filename)
                .part_number(part_number)
                .upload_id(upload_id.clone())
                .body(body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            // 保存etag
            let cp = CompletedPart::builder()
                .part_number(part_number)
                .e_tag(rsp.e_tag.unwrap())
                .build();
            complete_parts = complete_parts.parts(cp);

            uploaded += read_size as u64;
            part_number += 1;
        }

        // 结束上传
        let _rsp = self.client
            .complete_multipart_upload()
            .bucket(&self.config.bucket)
            .key(filename)
            .upload_id(upload_id)
            .multipart_upload(complete_parts.build())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
    
    async fn delete(&mut self, filename: &str) -> Result<(), String> {
        let _result = self.client
            .delete_object()
            .bucket(&self.config.bucket)
            .key(filename)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
