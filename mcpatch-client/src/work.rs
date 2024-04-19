use std::path::Path;
use std::time::SystemTime;

use mcpatch_shared::common::file_hash::calculate_hash_async;
use mcpatch_shared::data::index_file::IndexFile;
use mcpatch_shared::data::version_meta::FileChange;
use mcpatch_shared::data::version_meta::VersionMeta;
use mcpatch_shared::utility::vec_ext::VecRemoveIf;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;

use crate::error::BusinessError;
use crate::global_config::GlobalConfig;
use crate::log::log_debug;
use crate::network::Network;
use crate::utility::join_string;

pub async fn work(_work_dir: &Path, exe_dir: &Path, base_dir: &Path, config: &GlobalConfig, log_file_path: &Path) -> Result<(), BusinessError> {
    let mut network = Network::new(config);

    let version_file = exe_dir.join(&config.version_file_path);
    let current_version = tokio::fs::read_to_string(&version_file).await.unwrap_or("".to_owned());

    let server_versions = network.request_text("index.json", 0..0).await.unwrap();
    let server_versions = IndexFile::load_from_json(&server_versions);

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
    if !server_versions.contains(&current_version) {
        return Err(BusinessError::new("目前无法更新，因为客户端版本号不在服务端版本号列表里，无法确定版本新旧关系"));
    }

    // 不是最新版才更新
    let latest_version = &server_versions[server_versions.len() - 1].label;

    if latest_version != &current_version {
        // 收集落后的版本
        let missing_versions = (&server_versions).into_iter()
            .skip_while(|v| v.label == current_version)
            .collect::<Vec<_>>();

        log_debug("missing versions:");
        for i in 0..missing_versions.len() {
            log_debug(format!("  {}. {}", i, missing_versions[i].label));
        }
        
        // 下载所有更新包元数据
        let mut version_metas = Vec::<VersionMeta>::new();

        for ver in &missing_versions {
            let range = ver.offset..(ver.offset + ver.len as u64);
            let meta_text = network.request_text(&ver.filename, range).await.unwrap();
            let meta = json::parse(&meta_text).unwrap();

            for meta in meta.members().map(|e| VersionMeta::load(e)) {
                version_metas.push(meta);
            }
        }

        // 将多个文件变动列表合并成一个，并且尽可能剔除掉刚下载又马上要被删的文件，提高更新效率
        struct UpdateFile {
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

        for meta in &version_metas {
            for change in &meta.changes {
                match change.clone() {
                    FileChange::CreateFolder { path } => 
                        create_folders.push(path),
                    FileChange::UpdateFile { path, hash, len, modified, offset } => 
                        update_files.push(UpdateFile { path, hash, len, modified, offset }),
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
        for UpdateFile { path: raw_path, hash, len, modified, offset } in &update_files {
            let path = base_dir.join(&raw_path);
            let temp_path = base_dir.join(&format!("{}.temp", &raw_path));
            
            if path.exists() {
                let meta = tokio::fs::metadata(&path).await.unwrap();

                // 目标文件已经是目录了，就不要删除了，直接跳过，避免丢失玩家的数据
                if meta.is_dir() {
                    continue;
                }

                // 可以跳过更新，todo: 这里判断会有精度问题
                if meta.modified().unwrap() == *modified && meta.len() == *len {
                    continue;
                }

                // 对比hash，如果相同也可以跳过更新
                let mut f = tokio::fs::File::open(&path).await.unwrap();

                if calculate_hash_async(&mut f).await == *hash {
                    continue;
                }
            }
            
            // 发起请求
            let mut temp_file = tokio::fs::File::options().create(true).truncate(true).write(true).open(&temp_path).await.unwrap();
            let mut stream = network.request_file(&raw_path, *offset..(offset + len)).await.unwrap();
            let mut buf = [0u8; 16 * 1024];

            loop {
                let read = stream.1.read(&mut buf).await.unwrap();

                if read == 0 {
                    break;
                }

                temp_file.write_all(&buf[0..read]).await.unwrap();
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
        for MoveFile { from, to } in move_files {
            let from = base_dir.join(&from);
            let to = base_dir.join(&to);

            if from.exists() {
                tokio::fs::rename(from, to).await.unwrap();
            }
        }

        // 3.处理要删除的文件
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
        for path in create_folders {
            let path = base_dir.join(&path);

            tokio::fs::create_dir_all(path).await.unwrap();
        }

        // 6.合并临时文件
        // todo: 这里要给用户加提示，不能关程序
        for u in update_files {
            let path = base_dir.join(&u.path);
            let temp_path = base_dir.join(&format!("{}.temp", &u.path));

            tokio::fs::rename(temp_path, path).await.unwrap();
        }

        // 文件基本上更新完了，到这里就要进行收尾工作了
        // 1.更新客户端版本号
        tokio::fs::write(&version_file, latest_version.as_bytes()).await.unwrap();

        // 2.弹出更新记录
        println!("更新成功: {}", join_string(missing_versions.iter().map(|e| &e.label), ", "));
    }


    Ok(())
}
