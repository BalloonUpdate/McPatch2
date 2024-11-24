use std::time::Duration;
use std::time::SystemTime;

use rand::seq::SliceRandom;
use shared::utility::is_running_under_cargo;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub struct Token {
    pub token: String,
    pub expire: SystemTime,
}

impl Token {
    pub fn regen(&mut self) -> &str {
        self.expire = SystemTime::now().checked_add(Duration::from_secs(6 * 60 * 60)).unwrap();

        let mut rng = rand::rngs::OsRng;
        self.token = CHARSET.choose_multiple(&mut rng, 32).map(|e| *e as char).collect();

        &self.token
    }

    pub fn clear(&mut self) {
        self.token = "".to_owned();
    }

    pub fn validate(&self, token: &impl AsRef<str>) -> Result<(), &'static str> {
        let token = token.as_ref();
        let now = SystemTime::now();

        // 在开发期间可以使用空token跳过所有检查。但前提是没有调用登录api重新生成过token
        if is_running_under_cargo() {
            if token.is_empty() {
                return Ok(());
            }
        }

        // 检查token是否存在
        if self.token.is_empty() {
            return Err("empty token");
        }

        // 检查token是否有效
        if self.token != token {
            return Err("invalid token");
        }

        // 检查token是否过期
        if self.expire.duration_since(now).is_err() {
            return Err("token expired");
        }

        Ok(())
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            token: "".to_owned(), 
            expire: SystemTime::UNIX_EPOCH,
        }
    }
}