use std::ops::Deref;
use std::sync::Arc;
use std::time::SystemTime;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use tokio::sync::Mutex;

use crate::app_path::AppPath;

/// 用户认证相关配置
#[derive(Clone)]
pub struct AuthConfig {
    app_path: AppPath,
    inner: Arc<Mutex<Inner>>,
}

impl AuthConfig {
    pub async fn load(app_path: AppPath) -> (Self, Option<String>) {
        let exist = tokio::fs::try_exists(&app_path.auth_file).await.unwrap();

        if exist {
            let content = std::fs::read_to_string(&app_path.auth_file).unwrap();
            let data = toml::from_str::<Inner>(&content).unwrap();

            return (Self { app_path, inner: Arc::new(Mutex::new(data)) }, None);
        }

        let password = random_password();

        let inner = Inner::new(password.clone());

        let this = Self { app_path, inner: Arc::new(Mutex::new(inner)) };

        this.save().await;

        return (this, Some(password));
    }

    pub async fn set_username(&mut self, username: &str) {
        let mut lock = self.inner.lock().await;

        lock.username = username.to_owned();
    }

    pub async fn set_password(&mut self, password: &str) {
        let mut lock = self.inner.lock().await;

        lock.password = hash(password);
    }

    pub async fn test_username(&self, username: &str) -> bool {
        let lock = self.inner.lock().await;

        lock.username == username
    }

    pub async fn test_password(&self, password: &str) -> bool {
        let lock = self.inner.lock().await;

        lock.password == hash(password)
    }

    pub async fn regen_token(&mut self) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

        let mut lock = self.inner.lock().await;
        
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        // 有效期6个小时
        lock.expire = now + 6 * 60 * 60;

        let mut rng = rand::rngs::OsRng;
        let new_token: String = CHARSET.choose_multiple(&mut rng, 32).map(|e| *e as char).collect();

        lock.token = hash(&new_token);

        new_token
    }

    pub async fn clear_token(&mut self) {
        let mut lock = self.inner.lock().await;

        lock.token = "".to_owned();
    }

    pub async fn validate_token(&self, token: &str) -> Result<(), &'static str> {
        let lock = self.inner.lock().await;

        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        // 检查token是否存在
        if lock.token.is_empty() {
            return Err("empty token");
        }

        // 检查token是否有效
        if lock.token != hash(token) {
            return Err("invalid token");
        }

        // 检查token是否过期
        if lock.expire < now {
            return Err("token expired");
        }

        Ok(())
    }

    pub async fn save(&self) {
        let lock = self.inner.lock().await;

        let content = toml::to_string_pretty(lock.deref()).unwrap();

        std::fs::write(&self.app_path.auth_file, content).unwrap();
    }

    pub async fn username(&self) -> String {
        let lock = self.inner.lock().await;

        lock.username.to_owned()
    }

    pub async fn password(&self) -> String {
        let lock = self.inner.lock().await;

        lock.password.to_owned()
    }
}

/// 用户认证相关配置
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Inner {
    /// 用户名
    pub username: String,

    /// 密码的hash，计算方法：sha256(password)
    pub password: String,

    /// 目前保存的token的hash
    pub token: String,

    /// token的到期时间
    pub expire: u64,
}

impl Inner {
    fn new(password: String) -> Self {
        Self {
            username: "admin".to_owned(), 
            password: hash(&password),
            token: "".to_owned(),
            expire: 0,
        }
    }
}

fn hash(text: &str) -> String {
    let hash = Sha256::digest(text);
    
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