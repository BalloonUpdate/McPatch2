use std::path::PathBuf;

use crate::utility::is_running_under_cargo;

/// 代表各种目录的信息
#[derive(Clone)]
pub struct AppPath {
    /// 工作目录
    pub working_dir: PathBuf,

    /// 工作空间目录。用来放置要参与更新的文件
    pub workspace_dir: PathBuf,

    /// 公共目录。用来存放更新包向外提供服务
    pub public_dir: PathBuf,

    /// 外部加载的web目录。当这个目录存在时，会优先从这个目录加载web目录资源，然后是从可执行文件内部
    pub web_dir: PathBuf,

    /// 索引文件路径。用来识别当前有哪些更新包
    pub index_file: PathBuf,

    /// 配置文件路径。用来存储管理端的配置项目
    pub config_file: PathBuf,

    /// 认证数据文件路径。用来存储用户认证等数据
    pub auth_file: PathBuf,
}

impl AppPath {
    pub fn new() -> Self {
        let mut working_dir = std::env::current_dir().unwrap();
        
        // 在开发模式下，会将工作空间移动到test目录下方便测试
        if is_running_under_cargo() {
            working_dir = working_dir.join("test");
        }

        let workspace_dir = working_dir.join("workspace");
        let public_dir = working_dir.join("public");
        let web_dir = working_dir.join("webpage");
        let index_file = working_dir.join("public/index.json");
        let config_file = working_dir.join("config.toml");
        let auth_file = working_dir.join("user.toml");

        std::fs::create_dir_all(&workspace_dir).unwrap();
        std::fs::create_dir_all(&public_dir).unwrap();

        Self {
            working_dir,
            workspace_dir,
            public_dir,
            web_dir,
            index_file,
            config_file,
            auth_file,
        }
    }
}
