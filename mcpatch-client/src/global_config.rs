use std::path::Path;

use config_template_derive::ConfigTemplate;

#[derive(ConfigTemplate)]
pub struct GlobalConfig {
    /// 更新服务器上的索引文件的链接，可以填写多个备用链接以缓解网络不稳定
    /// 目前支持的协议：http(s)、webdav(s)、私有协议
    ///
    /// http协议例子：
    ///   1. http://127.0.0.1:6600/index.json （走http协议）
    ///   2. https://127.0.0.1:6600/subfolder/index.json （走https协议）
    ///
    /// webdav协议：（webdav代表走http协议，webdavs代表走https协议，这样写是为了和http源做区分）
    /// ```
    ///   1. webdav://user:pass:127.0.0.1:80 （默认形式，webdav使用http协议）
    ///   2. webdavs://user:pass:127.0.0.1:443/subfolder （子目录形式，webdav使用https协议，注意https默认端口为443，而非80）
    ///      -------   ---- ---- --------- --- ---------
    ///         |       |    |       |      |      |
    ///         |       |    |       |      |      +------ webdav 目录（可选）
    ///         |       |    |       |      +------------- webdav 端口（注意端口不能省略，通常是80和443
    ///         |       |    |       +-------------------- webdav 主机地址
    ///         |       |    +---------------------------- webdav 密码
    ///         |       +--------------------------------- webdav 用户名
    ///         +----------------------------------------- webdav 协议，只能是webdav或者webdavs
    /// ```
    #[default_value("\n  - http://127.0.0.1 # 若在公网部署记得换成自己的公网ip或者域名")]
    pub urls: Vec<String>,

    /// 记录客户端版本号文件的路径
    /// 客户端的版本号会被存储在这个文件里，并以此为依据判断是否更新到了最新版本
    #[default_value("mcpatch-version.txt")]
    pub version_file_path: String,

    /// 更新的起始目录，也就是要把文件都更新到哪个目录下
    /// 默认情况下程序会智能搜索，并将所有文件更新到.minecraft父目录下（也是启动主程序所在目录），
    /// 这样文件更新的位置就不会随主程序文件的工作目录变化而改变了，每次都会更新在相同目录下。
    /// 如果你不喜欢这个智能搜索的机制，可以修改此选项来把文件更新到别的地方（十分建议保持默认不要修改）
    /// 1. 当此选项的值是空字符串''时，会智能搜索.minecraft父目录作为更新起始目录（这也是默认值）
    /// 2. 当此选项的值是'.'时，会把当前工作目录作为更新起始目录
    /// 3. 当此选项的值是'..'时，会把当前工作目录的上级目录作为更新起始目录
    /// 4. 当此选项的值是别的时，比如'ab/cd'时，会把当前工作目录下的ab目录里面的cd目录作为更新起始目录
    #[default_value("''")]
    pub base_path: String,

    /// 当程序发生错误而更新失败时，是否可以继续进入游戏
    /// 如果为true，发生错误时会忽略错误，正常启动游戏，但是可能会因为某些新模组未下载无法进服
    /// 如果为false，发生错误时会直接崩溃掉Minecraft进程，停止游戏启动过程
    /// 此选项仅当程序以非图形模式启动时有效，因为在图形模式下，会主动弹框并将选择权交给用户
    #[default_value("false")]
    pub allow_error: bool,

    /// 安静模式，是否只在下载文件时才显示窗口
    /// 如果为true，程序启动后在后台静默检查文件更新，而不显示窗口，若没有更新会直接启动Minecraft，
    ///            有更新的话再显示下载进度条窗口，此选项可以尽可能将程序的存在感降低（适合线上环境）
    /// 如果为false，每次都正常显示窗口（适合调试环境）
    /// 此选项仅当程序以图形模式启动时有效
    #[default_value("false")]
    pub silent_mode: bool,

    /// 为http类协议设置headers，包括http(s)，webdav(s)
    #[default_value("\n#  User-Agent: This filled by youself # 这是一个自定义UserAgent的配置示例")]
    pub http_headers: Vec<(String, String)>,

    /// http类协议：连接超时判定时间，单位毫秒
    /// 网络环境较差时可能会频繁出现连接超时，那么此时可以考虑增加此值（建议30s以下）
    /// 建议连带 http_reading_timeout 选项一起修改，两边的值保持相同即可
    #[default_value("10000")]
    pub http_connection_timeout: u32,

    /// http类协议：读取超时判定时间，单位毫秒
    /// 网络环境较差时可能会频繁出现连接超时，那么此时可以考虑增加此值（建议30s以下）
    /// 建议连带 http_connection_timeout 选项一起修改，两边的值保持相同即可
    #[default_value("10000")]
    pub http_reading_timeout: u32,

    /// http类协议：重试次数，最大值不能超过255
    /// 当 http_connection_timeout 和 http_connection_timeout 的超时后，会消耗1次重试次数
    /// 当所有的重试次数消耗完后，程序才会真正判定为超时，并弹出网络错误对话框
    /// 建议总等待时长在60s以下，避免玩家等的太久：http_connection_timeout * http_retrying_times <= 60s
    #[default_value("3")]
    pub http_retrying_times: u8,

    /// http类协议：多线程下载时使用的线程数，最大值不能超过255
    /// 建议使用4线程下载
    #[default_value("4")]
    pub http_concurrent_threads: u8,

    /// http类协议：多线程下载时每个任务块的最大大小
    /// 建议保持默认值4194304(4mb)
    #[default_value("4194304")]
    pub http_concurrent_chunk_size: u32,

    /// http类协议：是否忽略SSL证书验证
    #[default_value("false")]
    pub http_ignore_certificate: bool,
}

impl GlobalConfig {
    pub fn load(file: &Path) -> Self {
        todo!()
    }
}
