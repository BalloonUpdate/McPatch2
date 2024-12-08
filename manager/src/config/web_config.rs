use serde::Deserialize;
use serde::Serialize;
use shared::utility::is_running_under_cargo;

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

    /// 控制`Access-Control-Allow-Credentials`的值
    pub cors_allow_credentials: bool,

    /// 控制`Access-Control-Allow-Headers`的值
    pub cors_allow_headers: Vec<String>,

    /// 控制`Access-Control-Allow-Methods`的值
    pub cors_allow_methods: Vec<String>,

    /// 控制`Access-Control-Allow-Origin`的值
    pub cors_allow_origin: Vec<String>,

    /// 控制`Access-Control-Allow-Private-Network`的值
    pub cors_allow_private_network: bool,

    /// 控制`Access-Control-Expose-Headers`的值
    pub cors_expose_headers: Vec<String>,

    pub coco: Option<bool>,
}

impl Default for WebConfig {
    fn default() -> Self {
        if is_running_under_cargo() {
            Self {
                listen_addr: "0.0.0.0".to_owned(), 
                listen_port: 6710, 
                tls_cert_file: "".to_owned(),
                tls_key_file: "".to_owned(),
                cors_allow_credentials: false,
                cors_allow_headers: vec!["*".to_owned()],
                cors_allow_methods: vec!["*".to_owned()],
                cors_allow_origin: vec!["*".to_owned()],
                cors_allow_private_network: false,
                cors_expose_headers: vec!["*".to_owned()],
                coco: None
            }
        } else {
            Self {
                listen_addr: "0.0.0.0".to_owned(), 
                listen_port: 6710,
                tls_cert_file: "".to_owned(),
                tls_key_file: "".to_owned(),
                cors_allow_credentials: false,
                cors_allow_headers: vec![],
                cors_allow_methods: vec![],
                cors_allow_origin: vec![],
                cors_allow_private_network: false,
                cors_expose_headers: vec![],
                coco: None
            }
        }
    }
}