use serde::Deserialize;
use serde::Serialize;

/// s3对象存储上传的配置
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub access_id: String,
    pub secret_key: String,
}