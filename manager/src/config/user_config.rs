use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

/// 用户认证相关配置
#[derive(Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "kebab-case")]
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