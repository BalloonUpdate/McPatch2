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
}

/// 核心功能配置（主要是打包相关）
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CoreConfig {
    /// 要排除的文件规则，格式为正则表达式，暂时不支持Glob表达式
    /// 匹配任意一条规则时，文件就会被忽略（忽略：管理端会当这个文件不存在一般）
    /// 编写规则时可以使用check命令快速调试是否生效
    pub exclude_rules: Vec<String>,
}

/// web相关功能配置
#[derive(Serialize, Deserialize, Clone)]
pub struct WebConfig {
    /// webui的监听地址
    pub serve_listen_addr: String,

    /// webui的监听端口
    pub serve_listen_port: u16,

    /// 用户名
    #[serde(default)]
    pub username: String,

    /// 密码的哈希值，计算方法：sha256(password)
    #[serde(default)]
    pub password_hash: String,
}

impl WebConfig {
    pub fn set_password(&mut self, password: &impl AsRef<str>) {
        self.password_hash = hash(password);
    }
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            serve_listen_addr: "0.0.0.0".to_owned(), 
            serve_listen_port: 6710, 
            username: "".to_owned(), 
            password_hash: hash(&random_password()),
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