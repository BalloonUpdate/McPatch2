use std::io::ErrorKind;
use std::path::Path;
use std::time::SystemTime;

use mcpatch_shared::common::file_hash::calculate_hash_async;
use mcpatch_shared::data::index_file::IndexFile;
use mcpatch_shared::data::version_meta::FileChange;
use mcpatch_shared::data::version_meta::VersionMeta;
use mcpatch_shared::utility::join_string;
use mcpatch_shared::utility::vec_ext::VecRemoveIf;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;

use crate::error::BusinessError;
use crate::global_config::GlobalConfig;
use crate::log::log_debug;
use crate::network::Network;
use crate::ui::AppWindowCommander;
use crate::ui::DialogContent;

pub async fn work(_work_dir: &Path, exe_dir: &Path, base_dir: &Path, config: &GlobalConfig, log_file_path: &Path, ui_cmd: &mut AppWindowCommander) -> Result<(), BusinessError> {
    let mut network = Network::new(config);

    let version_file = exe_dir.join(&config.version_file_path);
    let mut current_version = tokio::fs::read_to_string(&version_file).await.unwrap_or(":empty:".to_owned());

    if current_version.trim().is_empty() {
        current_version = ":empty:".to_owned();
    }
    
    ui_cmd.set_label("正在检查更新".to_owned()).await;

    let server_versions = network.request_text("index.json", 0..0, "index file").await.unwrap();
    let server_versions = IndexFile::load_from_json(&server_versions);

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
    if !server_versions.contains(&current_version) && current_version != ":empty:" {
        return Err(format!("目前无法更新，因为客户端版本号 {} 不在服务端版本号列表里，无法确定版本新旧关系", current_version).into());
    }

    // 不是最新版才更新
    let latest_version = &server_versions[server_versions.len() - 1].label;

    if latest_version != &current_version {
        if config.silent_mode {
            ui_cmd.set_visible(true).await;
        }

        // 收集落后的版本
        let missing_versions = (&server_versions).into_iter()
            .skip_while(|v| v.label == current_version)
            .collect::<Vec<_>>();

        log_debug("missing versions:");
        for i in 0..missing_versions.len() {
            log_debug(format!("  {}. {}", i, missing_versions[i].label));
        }
        
        // 下载所有更新包元数据
        let mut version_metas = Vec::<FullVersionMeta>::new();
        let mut counter = 1;

        for ver in &missing_versions {
            ui_cmd.set_label(format!("正在下载元数据 {} ({}/{})", ver.label, counter, missing_versions.len())).await;
            counter += 1;

            let range = ver.offset..(ver.offset + ver.len as u64);
            let meta_text = network.request_text(&ver.filename, range, format!("metadata of {}", ver.label)).await.unwrap();

            // println!("meta: <{}> {}", meta_text, meta_text.len());

            let meta = json::parse(&meta_text).unwrap();

            for meta in meta.members().map(|e| VersionMeta::load(e)) {
                version_metas.push(FullVersionMeta { filename: ver.filename.to_owned(), metadata: meta });
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

        ui_cmd.set_label("正在收集要更新的文件".to_owned()).await;

        for meta in &version_metas {
            for change in &meta.metadata.changes {
                match change.clone() {
                    FileChange::CreateFolder { path } => 
                        create_folders.push(path),
                    FileChange::UpdateFile { path, hash, len, modified, offset } => 
                        update_files.push(UpdateFile { 
                            package: meta.filename.to_owned(), 
                            label: meta.metadata.label.to_owned(), 
                            path, hash, len, modified, offset 
                        }),
                    FileChange::DeleteFolder { path } => 
                        delete_folders.push(path),
                    FileChange::DeleteFile { path } => {
                        // 处理哪些刚下载又马上要被删的文件
                        match update_files.iter().position(|e| e.path == path) {
                            Some(index) => { update_files.remove(index); },
                            None => delete_files.push(path),
                        }
                    },
                    FileChange::MoveFile { from, to } => 
                        move_files.push(MoveFile { from, to }),
                }
            }
        }

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
        
        // 执行更新流程
        // 1.处理要下载的文件，下载到临时文件
        let temp_dir = base_dir.join(".mcpatch-temp");

        // 创建临时文件夹
        if !update_files.is_empty() {
            tokio::fs::create_dir_all(&temp_dir).await.unwrap();
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
                    if meta.modified().unwrap() == f.modified && meta.len() == f.len {
                        update_files.remove(i);
                        continue;
                    }

                    // 对比hash，如果相同也可以跳过更新
                    let mut open = tokio::fs::File::open(&target_path).await.unwrap();

                    if calculate_hash_async(&mut open).await == f.hash {
                        update_files.remove(i);
                        continue;
                    }
                },
                Err(e) => {
                    if e.kind() != ErrorKind::NotFound {
                        Result::<std::fs::Metadata, std::io::Error>::Err(e).unwrap();
                    }
                },
            }
        }

        ui_cmd.set_label("下载更新数据".to_owned()).await;
        // tokio::time::sleep(std::time::Duration::from_millis(500000)).await;

        let mut total_bytes = 0u64;
        let mut total_downloaded = 0u64;

        for u in &update_files {
            total_bytes += u.len;
        }

        let mut file_counter = 0;

        // 下载到临时文件
        for UpdateFile { package, label, path, hash, len, modified: _, offset } in &update_files {
            let temp_path = temp_dir.join(&format!("{}.temp", &path));
            
            // println!("download to {:?}", temp_path);

            tokio::fs::create_dir_all(temp_path.parent().unwrap()).await.unwrap();

            file_counter += 1;
            ui_cmd.set_label(format!("下载版本 {} 的更新数据 ({}/{})", label, file_counter, update_files.len())).await;
            ui_cmd.set_label_secondary(format!("{}", path)).await;

            // 发起请求
            let mut temp_file = tokio::fs::File::options().create(true).truncate(true).read(true).write(true).open(&temp_path).await.unwrap();
            let mut stream = network.request_file(package, *offset..(offset + len), format!("{} in {}", path, label)).await.unwrap();
            let mut buf = [0u8; 16 * 1024];
            let mut bytes_counter = 0u64;

            loop {
                let read = stream.1.read(&mut buf).await.unwrap();

                if read == 0 {
                    break;
                }

                temp_file.write_all(&buf[0..read]).await.unwrap();

                bytes_counter += read as u64;
                total_downloaded += read as u64;
                
                // tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                
                ui_cmd.set_progress(((total_downloaded as f32 / total_bytes as f32) * 1000f32) as u32).await;
                ui_cmd.set_label_secondary(format!("{} ({:.1}%)", path, ((bytes_counter as f32 / *len as f32) * 100f32))).await;
            }

            // 检查下载的文件的hash对不对
            temp_file.flush().await.unwrap();
            temp_file.seek(std::io::SeekFrom::Start(0)).await.unwrap();

            let temp_hash = calculate_hash_async(&mut temp_file).await;

            if &temp_hash != hash {
                return Err(format!("the temp file hash {} does not match {}", &temp_hash, hash).into());
            }
        }
        
        // 2.处理要移动的文件
        ui_cmd.set_label("正在处理文件移动".to_owned()).await;

        for MoveFile { from, to } in move_files {
            let from = base_dir.join(&from);
            let to = base_dir.join(&to);

            if from.exists() {
                tokio::fs::rename(from, to).await.unwrap();
            }
        }

        // 3.处理要删除的文件
        ui_cmd.set_label("正在处理旧文件和旧目录".to_owned()).await;

        for path in delete_files {
            let path = base_dir.join(&path);

            tokio::fs::remove_file(path).await.unwrap();
        }

        // 4.处理要删除的目录
        for path in delete_folders {
            let path = base_dir.join(&path);

            // 删除失败了不用管
            match tokio::fs::remove_dir(path).await {
                Ok(_) => {},
                Err(_) => {},
            }
        }

        // 5.处理要创建的空目录
        ui_cmd.set_label("正在处理新目录".to_owned()).await;

        for path in create_folders {
            let path = base_dir.join(&path);

            tokio::fs::create_dir_all(path).await.unwrap();
        }

        // 6.合并临时文件
        ui_cmd.set_label("正在合并临时文件，请不要关闭程序，避免数据损坏".to_owned()).await;

        for u in &update_files {
            let target_path = base_dir.join(&u.path);
            let temp_path = temp_dir.join(&format!("{}.temp", &u.path));

            tokio::fs::rename(temp_path, target_path).await.unwrap();
        }

        // 清理临时文件夹
        ui_cmd.set_label("正在清理临时文件夹".to_owned()).await;

        if temp_dir.exists() {
            // println!("removing: {:?}", &temp_dir);
            tokio::fs::remove_dir_all(&temp_dir).await.unwrap();
        }

        // 文件基本上更新完了，到这里就要进行收尾工作了
        ui_cmd.set_label("正在进行收尾工作".to_owned()).await;

        // 1.更新客户端版本号
        tokio::fs::write(&version_file, latest_version.as_bytes()).await.unwrap();

        // 2.弹出更新记录
        println!("更新成功: {}", join_string(missing_versions.iter().map(|e| &e.label), ", "));

        ui_cmd.popup_dialog(DialogContent {
            title: "更新成功".to_owned(),
            content: format!("更新成功: {}", join_string(missing_versions.iter().map(|e| &e.label), ", ")),
            yesno: false,
        }).await;
    } else {
        ui_cmd.set_label("没有更新".to_owned()).await;
    }

    Ok(())
}
