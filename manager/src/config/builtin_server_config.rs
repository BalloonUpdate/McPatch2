use serde::Deserialize;
use serde::Serialize;

/// 私有协议服务端相关配置
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