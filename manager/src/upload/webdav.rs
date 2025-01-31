use std::path::PathBuf;
use std::time::Duration;

use reqwest_dav::list_cmd::ListEntity;
use reqwest_dav::Client;
use reqwest_dav::ClientBuilder;
use reqwest_dav::Depth;

use crate::config::webdav_config::WebdavConfig;
use crate::upload::UploadTarget;
use crate::utility::to_detail_error::ToDetailError;

pub struct WebdavTarget {
    _config: WebdavConfig,
    client: Client,
}

impl WebdavTarget {
    pub async fn new(config: WebdavConfig) -> Self {
        let reqwest_client = reqwest_dav::re_exports::reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_millis(10000 as u64))
            .read_timeout(Duration::from_millis(10000 as u64))
            // .danger_accept_invalid_certs(config.http_ignore_certificate)
            .use_rustls_tls() // https://github.com/seanmonstar/reqwest/issues/2004#issuecomment-2180557375
            .build()
            .unwrap();

        let client = ClientBuilder::new()
            .set_agent(reqwest_client)
            .set_host(config.host.clone())
            .set_auth(reqwest_dav::Auth::Basic(config.username.clone(), config.password.clone()))
            .build()
            .unwrap();

        Self {
            _config: config,
            client,
        }
    }
}

impl UploadTarget for WebdavTarget {
    async fn list(&mut self) -> Result<Vec<(String, u64)>, String> {
        let items = self.client.list("", Depth::Number(1)).await
            .map_err(|e| e.to_detail_error())?;
        
        let mut files = Vec::new();

        for item in items {
            if let ListEntity::File(file) = item {
                files.push((file.href, file.last_modified.timestamp() as u64));
            }
        }

        Ok(files)
    }

    async fn read(&mut self, filename: &str) -> Result<Option<String>, String> {
        let rsp = match self.client.get(&format!("/{}", filename)).await {
            Ok(ok) => ok,
            Err(err) => {
                if let reqwest_dav::types::Error::Decode(err) = &err {
                    if let reqwest_dav::types::DecodeError::Server(err) = err {
                        if err.response_code == 404 {
                            return Ok(None);
                        }
                    }
                }
                
                return Err(err.to_detail_error());
            },
        };

        Ok(Some(rsp.text().await.unwrap()))
    }

    async fn write(&mut self, filename: &str, content: &str) -> Result<(), String> {
        self.client.put(filename, content.to_owned()).await
            .map_err(|e| e.to_detail_error())?;
        
        Ok(())
    }

    async fn upload(&mut self, filename: &str, filepath: PathBuf) -> Result<(), String> {
        let file = tokio::fs::File::open(filepath).await.unwrap();

        self.client.put(filename, file).await
            .map_err(|e| e.to_detail_error())?;

        Ok(())
    }

    async fn delete(&mut self, filename: &str) -> Result<(), String> {
        self.client.delete(filename).await
            .map_err(|e| e.to_detail_error())?;

        Ok(())
    }
}