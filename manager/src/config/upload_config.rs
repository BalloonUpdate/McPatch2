use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct UploadConfig {
    pub bucket: String,
    pub region: String,
    pub access_id: String,
    pub secret_key: String,
}