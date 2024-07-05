use std::path::Path;

use config_template_derive::ConfigTemplate;

use crate::error::BusinessResult;
use crate::error::OptionToBusinessError;
use crate::error::ResultToBusinessError;

#[derive(ConfigTemplate)]
pub struct GlobalConfig {
    /// 更新服务器地址，可以填写多个备用地址，当一个不可用时会切换到备用地址上
    /// 目前支持的协议：http(s)、webdav(s)、私有协议
    ///
    /// http协议的例子：（填写索引文件index.json所在的目录就好，不需要填写index.json本身）
    ///   1. http://127.0.0.1:6700 （走http协议）
    ///   2. https://127.0.0.1:6700/subfolder （走https协议）
    ///
    /// webdav协议的例子：（webdav代表走http协议，webdavs代表走https协议，要这样写只是为了和http源做区分）
    ///   1. webdav://user:pass:127.0.0.1:80   （webdav走http协议）
    ///   2. webdavs://user:pass:127.0.0.1:443 （webdav走https协议）
    ///   注：需要把user和pass这两个地方换成自己的账号密码，127.0.0.1换成主机地址，端口号不能省略
    /// 
    /// 私有协议的例子：（私有协议是mcpatch自己的协议，无需备案，如果做内网穿透请走普通tcp隧道而非http隧道）
    ///   1. mcpatch://127.0.0.1:6700 （私有协议以mcpatch开头，只需要主机和端口号即可，无需输入子目录）
    /// 
    #[default_value("\n  - mcpatch://127.0.0.1:6700 # 若在公网部署记得换成自己的公网ip或者域名")]
    pub urls: Vec<String>,

    /// 记录客户端版本号文件的路径
    /// 客户端的版本号会被存储在这个文件里，并以此为依据判断是否更新到了最新版本
    #[default_value("version-label.txt")]
    pub version_file_path: String,

    /// 当程序发生错误而更新失败时，是否可以继续进入游戏
    /// 如果为true，发生错误时会忽略错误，正常启动游戏，但是可能会因为某些新模组未下载无法进服
    /// 如果为false，发生错误时会直接崩溃掉Minecraft进程，停止游戏启动过程
    /// 此选项仅当程序以非图形模式启动时有效，因为在图形模式下，会主动弹框并将选择权交给用户
    #[default_value("false")]
    pub allow_error: bool,

    /// 在没有更新时，是否显示“资源文件暂无更新!”提示框
    /// 在有更新时，此选项不生效
    #[default_value("true")]
    pub show_finish_message: bool,

    /// 安静模式，是否只在下载文件时才显示窗口
    /// 如果为true，程序启动后在后台静默检查文件更新，而不显示窗口，若没有更新会直接启动Minecraft，
    ///            有更新的话再显示下载进度条窗口，此选项可以尽可能将程序的存在感降低（适合线上环境）
    /// 如果为false，每次都正常显示窗口（适合调试环境）
    /// 此选项仅当程序以图形模式启动时有效
    #[default_value("false")]
    pub silent_mode: bool,

    /// 窗口标题，可以自定义更新时的窗口标题
    /// 只有在桌面环境上时才有效，因为非桌面环境没法弹出窗口
    #[default_value("Mcpatch")]
    pub window_title: String,

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

    /// 私有协议的超时判定时间，单位毫秒，值越小判定越严格
    /// 网络环境较差时可能会频繁出现连接超时，那么此时可以考虑增加此值（建议30s以下）
    #[default_value("7000")]
    pub private_timeout: u32,

    /// 为http/webdav设置协议头
    #[default_value("\n#  User-Agent: This filled by youself # 这是一个自定义UserAgent的配置示例")]
    pub http_headers: Vec<(String, String)>,

    /// http/webdav协议的连接超时判定时间，单位毫秒，值越小判定越严格
    /// 网络环境较差时可能会频繁出现连接超时，那么此时可以考虑增加此值（建议30s以下）
    #[default_value("5000")]
    pub http_timeout: u32,

    /// http/webdav协议的重试次数，最大值为254
    /// 当超过http_timeout服务器还是没有响应数据时，会消耗1次重试次数，然后进行重新连接
    /// 当所有的重试次数消耗完后，程序才会真正判定为超时，并弹出网络错误对话框
    /// 建议 http_timeout * http_retries 在20秒以内，避免玩家等的太久
    #[default_value("3")]
    pub http_retries: u8,

    /// http/webdav协议是否忽略SSL证书验证
    #[default_value("false")]
    pub http_ignore_certificate: bool,
}

impl GlobalConfig {
    pub async fn load(file: &Path) -> BusinessResult<Self> {
        let mut config = yaml_rust::yaml::Hash::new();

        // 生成默认的配置文件
        if !file.exists() {
            tokio::fs::write(&file, GlobalConfigTemplate).await
                .be(|e| format!("生成默认配置文件失败({:?})，原因：{}", file, e))?;
        }

        // 读取配置文件
        let content = tokio::fs::read_to_string(file).await
            .be(|e| format!("读取配置文件失败({:?})，原因：{}", file, e))?;
        let first = yaml_rust::YamlLoader::load_from_str(&content)
            .be(|e| format!("配置文件解析失败({:?})，原因：{}", file, e))?
            .remove(0);

        for (k ,v) in first.into_hash().unwrap() {
            config.insert(k, v);
        }

        // 补全默认配置
        let default = yaml_rust::YamlLoader::load_from_str(GlobalConfigTemplate)
            .be(|e| format!("配置文件模板解析失败，原因：{}", e))?
            .remove(0);
        
        for (k ,v) in default.into_hash().be(|| "配置文件模板格式不正确")? {
            if !config.contains_key(&k) {
                config.insert(k, v);
            }
        }

        let config = yaml_rust::Yaml::Hash(config);

        let urls = config["urls"].as_vec().be(|| "配置文件中找不到 urls")?.iter()
                .map(|e| e.as_str().expect("配置文件 urls 中只能包含纯字符串元素").to_owned())
                .collect();
        let version_file_path = config["version-file-path"].as_str().be(|| "配置文件中找不到 version-file-path")?.to_owned();
        let allow_error = config["allow-error"].as_bool().be(|| "配置文件中找不到 allow-error")?.to_owned();
        let show_finish_message = config["show-finish-message"].as_bool().be(|| "配置文件中找不到 show-finish-message")?.to_owned();
        let silent_mode = config["silent-mode"].as_bool().be(|| "配置文件中找不到 silent-mode")?.to_owned();
        let window_title = config["window-title"].as_str().be(|| "配置文件中找不到 window-title")?.to_owned();
        let base_path = config["base-path"].as_str().be(|| "配置文件中找不到 base-path")?.to_owned();
        let private_timeout = config["private-timeout"].as_i64().be(|| "配置文件中找不到 private-timeout")? as u32;
        let http_headers = match config["http-headers"].as_hash() {
            Some(map) => map.iter()
                .map(|e| {
                    let k = e.0.as_str().expect("http协议头中列表元素的 key 只能是字符串").to_owned();
                    let v = e.1.as_str().expect("http协议头中列表元素的 value 只能是字符串").to_owned();
                    (k, v)
                })
                .collect(),
            None => Vec::new(),
        };
        let http_timeout = config["http-timeout"].as_i64().be(|| "配置文件中找不到 http-timeout")? as u32;
        let http_retries = config["http-retries"].as_i64().be(|| "配置文件中找不到 http-retries")? as u8;
        let http_ignore_certificate = config["http-ignore-certificate"].as_bool().be(|| "配置文件中找不到 http-ignore-certificate")?.to_owned();

        Ok(GlobalConfig {
            urls,
            version_file_path,
            allow_error,
            show_finish_message,
            silent_mode,
            window_title,
            base_path,
            private_timeout,
            http_headers,
            http_timeout,
            http_retries,
            http_ignore_certificate,
        })
    }
}
