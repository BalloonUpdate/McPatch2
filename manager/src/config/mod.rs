use serde::Deserialize;
use serde::Serialize;

use crate::app_path::AppPath;
use crate::config::builtin_server_config::BuiltinServerConfig;
use crate::config::core_config::CoreConfig;
use crate::config::s3_config::S3Config;
use crate::config::web_config::WebConfig;
use crate::config::webdav_config::WebdavConfig;

pub mod core_config;
pub mod web_config;
pub mod auth_config;
pub mod builtin_server_config;
pub mod s3_config;
pub mod webdav_config;

/// 全局配置
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub core: CoreConfig,
    pub web: WebConfig,
    pub builtin_server: BuiltinServerConfig,
    pub s3: S3Config,
    pub webdav: WebdavConfig,
}

impl Config {
    pub async fn load(app_path: &AppPath) -> Self {
        let exist = tokio::fs::try_exists(&app_path.config_file).await.unwrap();

        // 生成默认配置文件
        if !exist {
            let content = toml::to_string_pretty(&Config::default()).unwrap();

            std::fs::write(&app_path.auth_file, content).unwrap();
        }

        let content = tokio::fs::read_to_string(&app_path.config_file).await.unwrap();

        let config = toml::from_str::<Config>(&content).unwrap();

        config
    }
}