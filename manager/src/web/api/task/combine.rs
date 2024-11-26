use std::collections::HashMap;
use std::collections::LinkedList;
use std::rc::Weak;

use axum::extract::State;
use axum::response::Response;
use shared::data::index_file::IndexFile;
use shared::data::index_file::VersionIndex;
use shared::data::version_meta::FileChange;
use shared::data::version_meta_group::VersionMetaGroup;

use crate::common::archive_tester::ArchiveTester;
use crate::common::tar_reader::TarReader;
use crate::common::tar_writer::TarWriter;
use crate::diff::history_file::HistoryFile;
use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

pub const COMBINED_FILENAME: &str = "combined.tar";

/// 代表新的合并包中的某个文件数据要从哪个旧包中复制过来
struct Location {
    /// 所在的版本
    pub label: String,

    /// 所在的tar包的文件名
    pub filename: String,

    /// 最原始的文件路径（不受后续移动操作的影响）
    pub path: String,

    /// tar包中的文件偏移
    pub offset: u64,

    /// 数据的长度
    pub len: u64,
}

// 执行更新包合并操作
pub async fn api_combine(State(state): State<WebState>) -> Response {
    state.clone().te.lock().await
        .try_schedule(move || do_combine(state)).await;

    PublicResponseBody::<()>::ok_no_data()
}

fn do_combine(state: WebState) {
    let config = state.config;
    let mut console = state.console.blocking_lock();
    
    let index_file = IndexFile::load_from_file(&config.index_file);

    // 执行合并前需要先测试一遍
    console.log_debug("正在执行合并前的解压测试");
    let mut tester = ArchiveTester::new();
    for v in &index_file {
        tester.feed(config.public_dir.join(&v.filename), v.offset, v.len);
    }
    tester.finish(|e| console.log_debug(format!("{}/{} 正在测试 {} 的 {} ({}+{})", e.index, e.total, e.label, e.path, e.offset, e.len))).unwrap();
    console.log_debug("测试通过，开始更新包合并流程");

    // 开始合并流程
    let versions_to_be_combined = (&index_file).into_iter()
        .filter(|e| e.filename != COMBINED_FILENAME)
        .collect::<LinkedList<_>>();

    if versions_to_be_combined.is_empty() {
        console.log_info("没有更新包可以合并");
        return;
    }

    console.log_debug("正在读取数据");
    
    let mut history = HistoryFile::new_dir("workspace_root", Weak::new());
    let mut data_locations = HashMap::<String, Location>::new();

    // 保留所有元数据，最后会合并写入tar包里
    let mut meta_group = VersionMetaGroup::new();

    // 记录所有读取的元数据，避免重复读取消耗时间
    let mut meta_cache_keys = Vec::<String>::new();

    // 读取现有更新包，并复现在history上
    for v in &index_file {
        // 跳过读取过的元数据
        let cache_key = format!("{}|{}|{}", v.filename, v.offset, v.len);

        if meta_cache_keys.contains(&cache_key) {
            continue;
        }

        meta_cache_keys.push(cache_key);

        // 开始正常读取
        let mut reader = TarReader::new(config.public_dir.join(&v.filename));
        let group = reader.read_metadata_group(v.offset, v.len);
        
        for meta in group.into_iter() {
            if meta_group.contains_meta(&meta.label) {
                continue;
            }
            
            history.replay_operations(&meta);
            
            // 记录所有文件的数据和来源
            for change in &meta.changes {
                match change {
                    FileChange::UpdateFile { path, offset, len, .. } => {
                        data_locations.insert(path.to_owned(), Location {
                            label: meta.label.clone(),
                            filename: v.filename.to_owned(),
                            path: path.to_owned(),
                            offset: *offset,
                            len: *len,
                        });
                    },
                    FileChange::DeleteFile { path } => {
                        data_locations.remove(path);
                    },
                    FileChange::MoveFile { from, to } => {
                        let hold = data_locations.remove(from).unwrap();
                        data_locations.insert(to.to_owned(), hold);
                    }
                    _ => (),
                }
            }

            meta_group.add_meta(meta);
        }
    }

    console.log_debug("正在合并数据");

    // 生成新的合并包
    let new_tar_file = config.public_dir.join("_combined.temp.tar");
    let mut writer = TarWriter::new(&new_tar_file);

    // 写入每个版本里的所有文件数据
    for (_, loc) in &data_locations {
        // 读取原tar包中的文件，然后复制到合并包中
        let mut reader = TarReader::new(config.public_dir.join(&loc.filename));
        let read = reader.open_file(loc.offset, loc.len);
        writer.add_file(read, loc.len, &loc.path, &loc.label);
    }

    console.log_debug("正在更新元数据");

    // 写入元数据
    let version_count = meta_group.0.len();
    let meta_loc = writer.finish(meta_group);

    // 更新索引文件
    let new_index_filepath = config.public_dir.join("_index.temp.json");
    let mut new_index_file = IndexFile::new();
    for index in &index_file {
        new_index_file.add(VersionIndex {
            label: index.label.to_owned(),
            filename: COMBINED_FILENAME.to_owned(),
            offset: meta_loc.offset,
            len: meta_loc.length,
            hash: "no hash".to_owned(),
        })
    }
    new_index_file.save(&new_index_filepath);

    // 测试合并包
    let mut tester = ArchiveTester::new();
    tester.feed(&new_tar_file, meta_loc.offset, meta_loc.length);
    tester.finish(|e| console.log_debug(format!("{}/{} 正在测试 {} 的 {} ({}+{})", e.index, e.total, e.label, e.path, e.offset, e.len))).unwrap();
    
    // 合并回原包
    std::fs::copy(&new_index_filepath, &config.index_file).unwrap();

    let combine_file = config.public_dir.join(COMBINED_FILENAME);
    let _ = std::fs::remove_file(&combine_file);
    std::fs::rename(&new_tar_file, &combine_file).unwrap();
    std::fs::remove_file(&new_index_filepath).unwrap();
    
    for v in &versions_to_be_combined {
        std::fs::remove_file(config.public_dir.join(&v.filename)).unwrap();
    }

    console.log_info(format!("合并完成！一共合并了 {} 个版本", version_count));

    // // 生成上传脚本
    // let context = TemplateContext {
    //     upload_files: vec![combine_file.strip_prefix(&ctx.working_dir).unwrap().to_str().unwrap().to_owned()],
    //     delete_files: versions_to_be_combined.iter().map(|e| {
    //         ctx.public_dir.join(&e.filename)
    //             .strip_prefix(&ctx.working_dir).unwrap()
    //             .to_str().unwrap()
    //             .to_owned()
    //     }).collect(),
    // };

    // generate_upload_script(context, ctx, "combined");
}
