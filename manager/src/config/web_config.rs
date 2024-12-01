use serde::Deserialize;
use serde::Serialize;

/// web相关功能配置
#[derive(Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "kebab-case")]
pub struct WebConfig {
    /// webui的监听地址
    pub listen_addr: String,

    /// webui的监听端口
    pub listen_port: u16,

    /// https的证书文件
    pub tls_cert_file: String,

    /// https的私钥文件
    pub tls_key_file: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0".to_owned(), 
            listen_port: 6710, 
            tls_cert_file: "".to_owned(),
            tls_key_file: "".to_owned(),
        }
    }
}