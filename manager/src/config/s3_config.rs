use serde::Deserialize;
use serde::Serialize;

/// s3对象存储上传的配置
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct S3Config {
    /// 启用webdav的上传
    pub enabled: bool,

    /// 端点地址
    pub endpoint: String,

    /// 桶名
    pub bucket: String,

    /// 地域
    pub region: String,
    
    /// 认证id
    pub access_id: String,

    /// 认证key
    pub secret_key: String,
}
