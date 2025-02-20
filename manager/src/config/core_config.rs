use serde::Deserialize;
use serde::Serialize;

/// 核心功能配置（主要是打包相关）
#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default, rename_all = "kebab-case")]
pub struct CoreConfig {
    /// 要排除的文件规则，格式为正则表达式，暂时不支持Glob表达式
    /// 匹配任意一条规则时，文件就会被忽略（忽略：管理端会当这个文件不存在一般）
    /// 编写规则时可以使用check命令快速调试是否生效
    pub exclude_rules: Vec<String>,

    /// 是否工作在webui模式下，还是在交互式命令行模式下
    pub webui_mode: bool,
}