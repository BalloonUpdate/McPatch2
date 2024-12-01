use serde::Deserialize;
use serde::Serialize;

use crate::config::builtin_server_config::BuiltinServerConfig;
use crate::config::core_config::CoreConfig;
use crate::config::user_config::UserConfig;
use crate::config::web_config::WebConfig;

/// 全局配置
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    pub core: CoreConfig,
    pub web: WebConfig,
    pub user: UserConfig,
    pub builtin_server: BuiltinServerConfig,
}