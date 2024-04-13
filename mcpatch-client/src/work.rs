use std::path::Path;

use mcpatch_shared::data::index_file::IndexFile;
use mcpatch_shared::data::version_meta::VersionMeta;

use crate::error::BusinessError;
use crate::global_config::GlobalConfig;
use crate::log::log_debug;
use crate::net::Network;

pub async fn work(work_dir: &Path, exe_dir: &Path, base_dir: &Path, config: &GlobalConfig) -> Result<(), BusinessError> {
    let network = Network::new();

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

        // 开始更新过程
        let mut version_metas = Vec::<VersionMeta>::new();

        // 下载所有更新包元数据
        for ver in missing_versions {
            let range = ver.offset..(ver.offset + ver.len as u64);
            let meta_text = network.request_text(&ver.filename, range).await.unwrap();
            let meta = json::parse(&meta_text).unwrap();

            for meta in meta.members().map(|e| VersionMeta::load(e)) {
                version_metas.push(meta);
            }
        }
        
        // todo: 按需下载
        


    }


    Ok(())
}

