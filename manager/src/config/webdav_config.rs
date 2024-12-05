use serde::Deserialize;
use serde::Serialize;

/// webdav上传的配置
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct WebdavConfig {
    /// 启用webdav的上传
    pub enabled: bool,

    /// 主机部分
    pub host: String,

    /// 用户名
    pub username: String,

    /// 密码
    pub password: String,
}
