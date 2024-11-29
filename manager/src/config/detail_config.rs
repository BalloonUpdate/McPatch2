use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

/// 全局配置
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    pub core: CoreConfig,
    pub web: WebConfig,
    pub user: UserConfig,
    pub builtin_server: BuiltinServerConfig,
}

/// 核心功能配置（主要是打包相关）
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CoreConfig {
    /// 要排除的文件规则，格式为正则表达式，暂时不支持Glob表达式
    /// 匹配任意一条规则时，文件就会被忽略（忽略：管理端会当这个文件不存在一般）
    /// 编写规则时可以使用check命令快速调试是否生效
    pub exclude_rules: Vec<String>,
}

/// web相关功能配置
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct UserConfig {
    /// 用户名
    #[serde(default)]
    pub username: String,

    /// 密码的哈希值，计算方法：sha256(password)
    #[serde(default)]
    pub password: String,

    // /// 目前保存的token
    // pub token: String,
}

impl UserConfig {
    pub fn set_password(&mut self, password: &impl AsRef<str>) {
        self.password = hash(password);
    }

    pub fn test_password(&self, password: &impl AsRef<str>) -> bool {
        self.password == hash(password)
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            username: "admin".to_owned(), 
            password: random_password(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct BuiltinServerConfig {
    /// 是否启动私有协议服务器功能
    pub enabled: bool,

    /// 私有协议服务器的监听地址
    pub listen_addr: String,

    /// 私有协议服务器的监听端口
    pub listen_port: u16,

    /// 内置服务端之限速功能的突发容量，单位为字节，默认为0不开启限速。
    /// 如果需要开启可以填写建议值1048576（背后的限速算法为令牌桶）
    pub capacity: u32,

    /// 内置服务端之限速功能的每秒回复的令牌数，单位为字节，默认为0不开启限速。
    /// 如果需要开启，这里填写需要限制的最大速度即可，比如1048576代表单链接限速1mb/s（背后的限速算法为令牌桶）
    pub regain: u32,
}

impl Default for BuiltinServerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            listen_addr: "0.0.0.0".to_owned(), 
            listen_port: 6700,
            capacity: 0,
            regain: 0,
        }
    }
}

fn hash(text: &impl AsRef<str>) -> String {
    let hash = Sha256::digest(text.as_ref());
    
    base16ct::lower::encode_string(&hash)
}

/// 生成一串随机的密码
fn random_password() -> String {
    const RAND_POOL: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();

    let mut password = String::new();

    for _ in 0..12 {
        let value = rng.gen_range(0..RAND_POOL.len());
        
        password.push(RAND_POOL[value] as char);
    }

    password
}