use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use shared::common::file_hash::calculate_hash_async;
use shared::data::index_file::IndexFile;
use shared::data::version_meta::FileChange;
use shared::data::version_meta::VersionMeta;
use shared::utility::filename_ext::GetFileNamePart;
use shared::utility::is_running_under_cargo;
use shared::utility::vec_ext::VecRemoveIf;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;

use crate::error::BusinessError;
use crate::error::BusinessResult;
use crate::error::OptionToBusinessError;
use crate::error::ResultToBusinessError;
use crate::global_config::GlobalConfig;
use crate::log::add_log_handler;
use crate::log::log_debug;
use crate::log::log_error;
use crate::log::log_info;
use crate::log::set_log_prefix;
use crate::log::ConsoleHandler;
use crate::log::FileHandler;
use crate::log::MessageLevel;
use crate::network::Network;
use crate::speed_sampler::SpeedCalculator;
use crate::utils::convert_bytes;
use crate::McpatchExitCode;
use crate::StartupParameter;

#[cfg(target_os = "windows")]
use crate::ui::message_box_ui::MessageBoxWindow;

#[cfg(target_os = "windows")]
use crate::ui::main_ui::DialogContent;

#[cfg(target_os = "windows")]
pub type UiCmd<'a> = &'a crate::ui::main_ui::MainUiCommand;

#[cfg(not(target_os = "windows"))]
pub type UiCmd<'a> = ();

pub async fn run(params: StartupParameter, ui_cmd: UiCmd<'_>) -> McpatchExitCode {
    // 初始化终端日志记录器
    let console_log_level = match is_running_under_cargo() {
        true => match params.graphic_mode || params.disable_log_file {
            true => MessageLevel::Debug,
            false => MessageLevel::Info, // 不需要太详细
        },
        false => MessageLevel::Debug,
    };
    add_log_handler(Box::new(ConsoleHandler::new(console_log_level)));

    let mut allow_error = false;

    // 将更新主逻辑单独拆到一个方法里以方便处理错误
    match work(&params, ui_cmd, &mut allow_error).await {
        Ok(_) => {
            log_info("finish");

            McpatchExitCode(0)
        },
        Err(e) => {
            log_error(&e.reason);

            if params.graphic_mode {
                #[cfg(target_os = "windows")]
                {
                    let choice = ui_cmd.popup_dialog(DialogContent {
                        title: "Error".to_owned(),
                        content: format!("{}\n\n确定：忽略错误继续启动\n取消：终止启动过程并报错", e.reason),
                        yesno: true,
                    }).await;

                    match choice {
                        true => McpatchExitCode(0),
                        false => McpatchExitCode(1),
                    }
                }

                #[cfg(not(target_os = "windows"))]
                match allow_error {
                    true => McpatchExitCode(0),
                    false => McpatchExitCode(1),
                }
            } else {
                match allow_error {
                    true => McpatchExitCode(0),
                    false => McpatchExitCode(1),
                }
            }
        },
    }
}

pub async fn work(params: &StartupParameter, ui_cmd: UiCmd<'_>, allow_error: &mut bool) -> Result<(), BusinessError> {
    let working_dir = get_working_dir(params).await?;
    let exe_dir = get_executable_dir(params).await?;
    let config = GlobalConfig::load(&exe_dir.join("mcpatch.yml")).await?;
    let base_dir = get_base_dir(params, &config).await?;

    *allow_error = config.allow_error;

    let log_file_path = match params.graphic_mode {
        true => exe_dir.join("mcpatch.log"),
        false => exe_dir.join("mcpatch.log.txt"),
    };

    // 显示窗口
    #[cfg(target_os = "windows")]
    if !config.silent_mode {
        ui_cmd.set_visible(true).await;
    }

    // 初始化窗口内容
    #[cfg(target_os = "windows")]
    {
        ui_cmd.set_title(config.window_title.to_owned()).await;
        ui_cmd.set_label("".to_owned()).await;
        ui_cmd.set_label_secondary("".to_owned()).await;    
    }

    // 初始化文件日志记录器
    if !params.disable_log_file {
        add_log_handler(Box::new(FileHandler::new(&log_file_path)));
    }

    // 没有独立进程的话需要加上日志前缀，好方便区分
    if !params.standalone_progress {
        set_log_prefix("Mcpatch");
    }

    // println!(">>>>>>>>>>>>>>>>>>>>>>>>>>> {:?}", base_dir);
    // tokio::fs::remove_dir_all(&base_dir).await.be(|e| format!("清理临时目录失败({:?})，原因：{}", base_dir, e))?;
    // tokio::fs::create_dir_all(&base_dir).await.be(|e| format!("创建新目录失败({:?})，原因：{}", base_dir, e))?;

    // 打印运行环境信息
    let gm = if params.graphic_mode { "yes" } else { "no" };
    let sp = if params.standalone_progress { "yes" } else { "no" };
    log_info(&format!("graphic_mode: {gm}, standalone_process: {sp}"));
    log_info(&format!("base directory: {}", base_dir.to_str().unwrap()));
    log_info(&format!("work directory: {}", working_dir.to_str().unwrap()));
    log_info(&format!("prog directory: {}", exe_dir.to_str().unwrap()));

    let mut network = Network::new(&config).be(|e| format!("服务器地址加载失败，原因：{:?}", e))?;

    let version_file = exe_dir.join(&config.version_file_path);
    let current_version = tokio::fs::read_to_string(&version_file).await.unwrap_or("".to_owned()).trim().to_owned();
    
    #[cfg(target_os = "windows")]
    ui_cmd.set_label("正在检查更新".to_owned()).await;

    let server_versions = network.request_text("index.json", 0..0, "index file").await.be(|e| format!("检查更新失败，原因：{:?}", e))?;
    let server_versions = IndexFile::load_from_json(&server_versions);

    #[cfg(target_os = "windows")]
    ui_cmd.set_label("正在看有没有更新".to_owned()).await;

    // 检查服务端版本数量
    if server_versions.len() == 0 {
        return Err(BusinessError::new("目前无法更新，因为服务端还没有打包任何更新包"));
    }

    // 输出服务端全部版本号
    log_debug("server versions:");
    for i in 0..server_versions.len() {
        log_debug(format!("  {}. {}", i, server_versions[i].label));
    }

    // 检查版本是否有效
    if !server_versions.contains(&current_version) && current_version != "" {
        return Err(format!("目前无法更新，因为客户端版本号 {} 不在服务端版本号列表里，无法确定版本新旧关系", current_version).into())
    }
    
    // 不是最新版才更新
    let latest_version = &server_versions[server_versions.len() - 1].label;
    
    println!("latest: {}, current: {}", latest_version, current_version);

    if latest_version != &current_version {
        if config.silent_mode {
            #[cfg(target_os = "windows")]
            ui_cmd.set_visible(true).await;
        }

        // 收集落后的版本
        let missing_versions = match (&server_versions).into_iter().position(|e| e.label == current_version) {
            Some(index) => {
                (&server_versions).into_iter().skip(index + 1).collect::<Vec<_>>()
            },
            // 搜索不到的话，current_version就是空字符串的情况
            None => {
                (&server_versions).into_iter().collect::<Vec<_>>()
            }, 
        };

        log_debug("missing versions:");
        for i in 0..missing_versions.len() {
            log_debug(format!("  {}. {}", i, missing_versions[i].label));
        }
        
        // 下载所有更新包元数据
        let mut version_metas = Vec::<FullVersionMeta>::new();
        let mut counter = 1;

        for ver in &missing_versions {
            #[cfg(target_os = "windows")]
            ui_cmd.set_label(format!("正在下载元数据 {} ({}/{})", ver.label, counter, missing_versions.len())).await;
            counter += 1;

            let range = ver.offset..(ver.offset + ver.len as u64);
            let meta_text = network.request_text(&ver.filename, range, format!("metadata of {}", ver.label)).await.be(|e| format!("元数据下载失败，原因：{:?}", e))?;

            // println!("meta: <{}> {}", meta_text, meta_text.len());

            let meta = json::parse(&meta_text).be(|e| format!("版本 {} 的元数据解析失败，原因：{:?}", ver.label, e))?;

            // 避免重复添加元数据
            for meta in meta.members().map(|e| VersionMeta::load(e)) {
                if version_metas.iter().find(|e| e.metadata.label == meta.label).is_none() {
                    version_metas.push(FullVersionMeta { filename: ver.filename.to_owned(), metadata: meta });
                }
            }
        }

        struct FullVersionMeta {
            /// 更新包文件名
            filename: String,

            /// 版本元数据
            metadata: VersionMeta
        }

        // 将多个文件变动列表合并成一个，并且尽可能剔除掉刚下载又马上要被删的文件，提高更新效率
        struct UpdateFile {
            /// 所属更新包文件名
            package: String,

            /// 所属版本号
            label: String,

            /// 要更新的文件路径
            path: String, 
    
            /// 文件校验值
            hash: String, 
            
            /// 文件长度
            len: u64, 
            
            /// 文件的修改时间
            modified: SystemTime, 

            /// 文件二进制数据在更新包中的偏移值
            offset: u64
        }

        struct MoveFile {
            /// 文件从哪里来
            from: String, 
            
            /// 文件到哪里去
            to: String
        }

        let mut create_folders = Vec::<String>::new();
        let mut update_files = Vec::<UpdateFile>::new();
        let mut delete_folders = Vec::<String>::new();
        let mut delete_files = Vec::<String>::new();
        let mut move_files = Vec::<MoveFile>::new();

        #[cfg(target_os = "windows")]
        ui_cmd.set_label("正在收集要更新的文件".to_owned()).await;

        for meta in &version_metas {
            for change in &meta.metadata.changes {
                match change.clone() {
                    FileChange::CreateFolder { path } => {
                        assert!(!create_folders.contains(&path));

                        // 先删除 delete_folders 里的文件夹。没有的话，再加入 create_folders 里面
                        match delete_folders.iter().position(|e| e == &path) {
                            Some(index) => { delete_folders.remove(index); },
                            None => { create_folders.push(path); },
                        }
                    },
                    FileChange::UpdateFile { path, hash, len, modified, offset } => {
                        // assert!(update_files.iter().find(|e| e.path == path).is_none());

                        // 删除已有的东西，避免下面重复添加报错
                        match update_files.iter().position(|e| e.path == path) {
                            Some(index) => { update_files.remove(index); },
                            None => { },
                        }

                        // 将文件从删除列表里移除
                        match delete_files.iter().position(|e| e == &path) {
                            Some(index) => { delete_files.remove(index); },
                            None => { },
                        }

                        update_files.push(UpdateFile { 
                            package: meta.filename.to_owned(), 
                            label: meta.metadata.label.to_owned(), 
                            path, hash, len, modified, offset 
                        });
                    },
                    FileChange::DeleteFolder { path } => {
                        assert!(!delete_folders.contains(&path));
                        
                        // 先删除 create_folders 里的文件夹。没有的话，再加入 delete_folders 里面
                        match create_folders.iter().position(|e| e == &path) {
                            Some(index) => { create_folders.remove(index); },
                            None => delete_folders.push(path),
                        }
                    },
                    FileChange::DeleteFile { path } => {
                        // 处理哪些刚下载又马上要被删的文件，这些文件不用重复下载
                        match update_files.iter().position(|e| e.path == path) {
                            Some(index) => { update_files.remove(index); },
                            None => { },
                        }

                        delete_files.push(path);
                    },
                    FileChange::MoveFile { from, to } => {
                        // 单独处理还没有下载的文件
                        match update_files.iter().position(|e| e.path == from) {
                            Some(index) => {
                                let removed = update_files.get_mut(index).unwrap();

                                // 目标文件不能存在
                                assert!(move_files.iter().find(|e| e.to == to).is_none());
                                
                                // 移动文件
                                removed.path = to;
                            },
                            None => {
                                // 处理存在的文件
                                assert!(move_files.iter().find(|e| e.from == from || e.to == to).is_none());

                                move_files.push(MoveFile { from, to });
                            },
                        }
                    },
                }
            }
        }

        // let mut cnt = 0;
        // for e in &update_files {
        //     println!("update_file({}): {}({})", cnt, e.path, e.label);
        //     cnt += 1;
        // }

        // 过滤一些不安全行为
        // 1.不能更新自己
        let current_exe = std::env::current_exe().unwrap();
        create_folders.remove_if(|e| base_dir.join(&e) == current_exe);
        update_files.remove_if(|e| base_dir.join(&e.path) == current_exe);
        delete_files.remove_if(|e| base_dir.join(&e) == current_exe);
        move_files.remove_if(|e| base_dir.join(&e.from) == current_exe || base_dir.join(&e.to) == current_exe);

        // 2.不能更新日志文件
        create_folders.remove_if(|e| base_dir.join(&e) == log_file_path);
        update_files.remove_if(|e| base_dir.join(&e.path) == log_file_path);
        delete_files.remove_if(|e| base_dir.join(&e) == log_file_path);
        move_files.remove_if(|e| base_dir.join(&e.from) == log_file_path || base_dir.join(&e.to) == log_file_path);

        for mf in &move_files {
            println!("move files: {} => {}", mf.from, mf.to);
        }
        
        // 执行更新流程
        // 1.处理要下载的文件，下载到临时文件
        let temp_dir = base_dir.join(".mcpatch-temp");

        // 创建临时文件夹
        if !update_files.is_empty() {
            tokio::fs::create_dir_all(&temp_dir).await.be(|e| format!("创建临时目录失败，原因：{:?}", e))?;
        }

        // 尽可能跳过要下载的文件
        for i in (0..update_files.len()).rev() {
            let f = &update_files[i];
            let target_path = base_dir.join(&f.path);

            // 检查一下看能不能跳过下载
            if !target_path.exists() {
                continue;
            }

            match tokio::fs::metadata(&target_path).await {
                Ok(meta) => {
                    // 目标文件已经是目录了，就不要删除了，直接跳过，避免丢失玩家的数据
                    if meta.is_dir() {
                        update_files.remove(i);
                        continue;
                    }

                    // 可以跳过更新，todo: 这里判断会有精度问题
                    let modified = meta.modified().be(|e| format!("获取文件修改失败失败({:?})，原因：{:?}", target_path, e))?;
                    if modified == f.modified && meta.len() == f.len {
                        update_files.remove(i);
                        continue;
                    }

                    // 对比hash，如果相同也可以跳过更新
                    let mut open = tokio::fs::File::open(&target_path).await
                        .be(|e| format!("打开文件失败({:?})，原因：{:?}", target_path, e))?;

                    if calculate_hash_async(&mut open).await == f.hash {
                        update_files.remove(i);
                        continue;
                    }
                },
                Err(e) => {
                    if e.kind() != ErrorKind::NotFound {
                        return Err(BusinessError::new(format!("获取文件metadata失败({:?})，原因：{:?}", target_path, e)));
                    }
                },
            }
        }

        #[cfg(target_os = "windows")]
        ui_cmd.set_label("下载更新数据".to_owned()).await;
        // tokio::time::sleep(std::time::Duration::from_millis(500000)).await;

        let mut total_bytes = 0u64;
        let mut total_downloaded = 0u64;

        for u in &update_files {
            total_bytes += u.len;
        }

        let mut _file_counter = 0;
        let mut speed = SpeedCalculator::new(1500);
        let mut ui_timer = SystemTime::now() - Duration::from_millis(600);

        // 下载到临时文件
        for UpdateFile { package, label, path, hash, len, modified: _, offset } in &update_files {
            let filename = Path::new(path).filename();
            let temp_path = temp_dir.join(&format!("{}.temp", &path));
            
            // println!("download to {:?}", temp_path);

            let temp_directory = temp_path.parent().be(|| format!("获取{:?}的上级目录失败，可能是抵达了文件系统根目录", temp_path))?;
            tokio::fs::create_dir_all(temp_directory).await.be(|e| format!("创建临时目录失败({:?})，原因：{:?}", temp_directory, e))?;

            _file_counter += 1;
            let now = SystemTime::now();
            if now.duration_since(ui_timer).unwrap().as_millis() > 100 {
                ui_timer = now;
                // ui_cmd.set_label(format!("下载版本 {} 的更新数据 ({}/{})", label, file_counter, update_files.len())).await;
                #[cfg(target_os = "windows")]
                {
                    ui_cmd.set_progress(((total_downloaded as f32 / total_bytes as f32) * 1000f32) as u32).await;
                    ui_cmd.set_label_secondary(format!("{}", filename)).await; // {:.1}% percent
                    ui_cmd.set_label(format!("更新 {} 版本：{}/{} （{}/s）", label, convert_bytes(total_downloaded), convert_bytes(total_bytes), speed.sample_speed2())).await;
                }
            }

            let mut temp_file = tokio::fs::File::options().create(true).truncate(true).read(true).write(true).open(&temp_path).await
                .be(|e| format!("打开临时文件失败({:?})，原因：{:?}", temp_path, e))?;

            // 开始下载
            let mut io_error = Option::<std::io::Error>::None;
            'outer: for i in 0..config.http_retries + 1 {
                temp_file.seek(std::io::SeekFrom::Start(0)).await.be(|e| format!("归零临时文件读写指针失败({:?})，原因：{:?}", temp_path, e))?;

                // 空文件不需要下载
                if *len == 0 {
                    break;
                }
                
                let (_, mut stream) = network.request_file(&package, *offset..(offset + len), &format!("{} in {}", path, label)).await.be(|e| format!("文件下载失败，原因：{:?}", e))?;

                let mut buf = [0u8; 32 * 1024];
                let mut bytes_counter = 0u64;

                loop {
                    let read = match stream.read(&mut buf).await {
                        Ok(read) => read,
                        Err(e) => {
                            io_error = Some(e);
                            total_downloaded -= bytes_counter;
                            if i != config.http_retries {
                                log_error("retrying")
                            }
                            continue 'outer;
                        },
                    };
        
                    if read == 0 {
                        break;
                    }
        
                    temp_file.write_all(&buf[0..read]).await.be(|e| format!("写入临时文件时失败(本地文件 {:?}, 远端信息: {} 里 {} 版本的 {})，原因：{:?}", temp_path, package, label, path, e))?;
        
                    bytes_counter += read as u64;
                    total_downloaded += read as u64;
                    speed.feed(read);
                    
                    // tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    
                    let now = SystemTime::now();
                    if now.duration_since(ui_timer).unwrap().as_millis() > 500 {
                        ui_timer = now;

                        #[cfg(target_os = "windows")]
                        {
                            // let percent = (bytes_counter as f32 / *len as f32) * 100f32;
                            ui_cmd.set_label(format!("正在下载 {} 版本：{}/{} （{}/s）", label, convert_bytes(total_downloaded), convert_bytes(total_bytes), speed.sample_speed2())).await;
                            // ui_cmd.set_label_secondary(format!("{}", temp_path.filename())).await; // {:.1}% percent
                            ui_cmd.set_progress(((total_downloaded as f32 / total_bytes as f32) * 1000f32) as u32).await;
                            // ui_cmd.set_title(format!("{} {}/s", config.window_title, spd)).await;
                        }
                    }
                }

                // 单文件下载进度
                // let now = SystemTime::now();
                // if now.duration_since(ui_timer).unwrap().as_millis() > 500 {
                //     ui_timer = now;
                //     ui_cmd.set_label_secondary(format!("{} ({:.1}%)", path, ((bytes_counter as f32 / *len as f32) * 100f32))).await;
                // }

                break;
            }

            if let Some(err) = io_error {
                return Err(BusinessError::new(format!("文件下载失败(本地文件 {:?}, 远端信息: {} 里 {} 版本的 {})，原因：{:?}", temp_path, package, label, path, err)));
            }

            // 检查下载的文件的hash对不对
            temp_file.flush().await.be(|e| format!("刷新临时文件失败({:?})，原因：{:?}", temp_path, e))?;
            temp_file.seek(std::io::SeekFrom::Start(0)).await.be(|e| format!("归零临时文件读写指针失败({:?})，原因：{:?}", temp_path, e))?;

            let temp_hash = calculate_hash_async(&mut temp_file).await;

            if &temp_hash != hash {
                return Err(format!("the temp file hash {} does not match {}", &temp_hash, hash).into());
            }
        }

        // 2.处理要创建的空目录
        #[cfg(target_os = "windows")]
        ui_cmd.set_label("正在处理新目录".to_owned()).await;

        for path in create_folders {
            log_debug(&format!("create directory: {}", path));

            let path = base_dir.join(&path);

            tokio::fs::create_dir_all(&path).await.be(|e| format!("创建新目录失败({:?})，原因：{:?}", path, e))?;
        }
        
        // 2.处理要移动的文件
        #[cfg(target_os = "windows")]
        ui_cmd.set_label("正在处理文件移动，请不要关闭程序".to_owned()).await;

        for MoveFile { from, to } in move_files {
            log_debug(&format!("move file {} => {}", from, to));

            let from = base_dir.join(&from);
            let to = base_dir.join(&to);

            if from.exists() {
                tokio::fs::rename(&from, &to).await.be(|e| format!("处理文件移动失败({:?} => {:?})，原因：{:?}", from, to, e))?;
            }
        }

        // 3.处理要删除的文件
        #[cfg(target_os = "windows")]
        ui_cmd.set_label("正在处理旧文件和旧目录".to_owned()).await;

        for path in delete_files {
            log_debug(&format!("delete file {}", path));

            let path = base_dir.join(&path);

            if tokio::fs::try_exists(&path).await.unwrap_or(false) {
                tokio::fs::remove_file(&path).await.be(|e| format!("删除旧文件失败({:?})，原因：{:?}", path, e))?;
            }
        }

        // 4.处理要删除的目录
        for path in delete_folders {
            log_debug(&format!("delete directory {}", path));

            let path = base_dir.join(&path);

            if let Err(e) = tokio::fs::remove_dir_all(path).await {
                log_error(format!("目录删除失败：{}", e));
            }
        }

        #[cfg(target_os = "windows")]
        ui_cmd.set_label("正在移动临时文件，请不要关闭程序".to_owned()).await;
        for u in &update_files {
            log_debug(&format!("apply temporary file {} => {}", &format!("{}.temp", &u.path), u.path));

            let target_path = base_dir.join(&u.path);
            let temp_path = temp_dir.join(&format!("{}.temp", &u.path));

            // println!("{} => {}", &u.path, format!("{}.temp", &u.path));
            
            tokio::fs::rename(&temp_path, &target_path).await.be(|e| format!("移动临时文件失败({:?} => {:?})，原因：{:?}", temp_path, target_path, e))?;
        }

        // 清理临时文件夹
        #[cfg(target_os = "windows")]
        ui_cmd.set_label("正在清理临时文件夹".to_owned()).await;

        if temp_dir.exists() {
            tokio::fs::remove_dir_all(&temp_dir).await.be(|e| format!("清理临时目录失败({:?})，原因：{:?}", temp_dir, e))?;
        }

        // 文件基本上更新完了，到这里就要进行收尾工作了
        #[cfg(target_os = "windows")]
        ui_cmd.set_label("正在进行收尾工作".to_owned()).await;

        // 1.更新客户端版本号
        tokio::fs::write(&version_file, latest_version.as_bytes()).await.be(|e| format!("更新客户端版本号文件为 {} 时失败({:?})，原因：{:?}", latest_version, version_file, e))?;

        // 2.弹出更新记录
        let mut changelogs = "".to_owned();

        for meta in &version_metas {
            changelogs += &format!("++++++++++ {} ++++++++++\n{}\n\n", meta.metadata.label, meta.metadata.logs);
        }

        log_info(format!("更新成功: \n{}", changelogs.trim()));

        // 弹出更新记录窗口
        #[cfg(target_os = "windows")]
        {
            let content = format!("已经从 {} 更新到 {}\r\n\r\n{}", current_version, latest_version, changelogs.trim().replace("\n", "\r\n"));

            if config.show_changelogs_message {
                MessageBoxWindow::popup(config.changelogs_window_title, content).await;
            }
        }
    } else {
        log_info("没有更新");

        #[cfg(target_os = "windows")]
        ui_cmd.set_label("没有更新".to_owned()).await;

        #[cfg(target_os = "windows")]
        if config.show_finish_message || !config.silent_mode {
            ui_cmd.popup_dialog(DialogContent {
                title: "".to_owned(),
                content: format!("当前已是最新的版本 {}", current_version),
                yesno: false,
            }).await;
        }
    }

    Ok(())
}

/// 获取更新起始目录
async fn get_base_dir(params: &StartupParameter, global_config: &GlobalConfig) -> BusinessResult<PathBuf> {
    let working_dir = get_working_dir(params).await?;

    if is_running_under_cargo() {
        return Ok(working_dir);
    }

    // 智能搜索.minecraft文件夹
    if global_config.base_path.is_empty() {
        let mut current = &working_dir as &Path;

        for _ in 0..7 {
            let detect = tokio::fs::try_exists(current.join(".minecraft")).await;

            match detect {
                Ok(found) => {
                    if found {
                        return Ok(current.to_owned());
                    }

                    current = match current.parent() {
                        Some(parent) => parent,
                        None => break,
                    }
                },
                Err(_) => break,
            }
        }

        return Err(BusinessError::new(".minecraft not found"));
    }

    let base_dir = working_dir.join(&global_config.base_path);
    tokio::fs::create_dir_all(&base_dir).await.be(|e| format!("创建更新起始目录失败({:?})，原因：{:?}", base_dir, e))?;
    Ok(base_dir)
}

/// 获取可执行文件所在目录
async fn get_executable_dir(params: &StartupParameter) -> BusinessResult<PathBuf> {
    if is_running_under_cargo() {
        get_working_dir(params).await
    } else {
        Ok(std::env::current_exe().be(|e| format!("获取exe文件路径失败，原因：{:?}", e))?.parent().unwrap().to_owned())
    }
}

/// 获取工作目录
async fn get_working_dir(_params: &StartupParameter) -> BusinessResult<PathBuf> {
    let mut working_dir = std::env::current_dir().be(|e| format!("获取工作目录失败，原因：{:?}", e))?;
        
    if is_running_under_cargo() {
        working_dir = working_dir.join("test").join("client");
    }

    tokio::fs::create_dir_all(&working_dir).await
        .be(|e| format!("创建工作目录失败，原因：{:?}", e))?;

    Ok(working_dir)
}

