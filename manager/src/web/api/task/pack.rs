use std::ops::Deref;
use std::rc::Weak;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::Json;
use serde::Deserialize;
use shared::data::index_file::IndexFile;
use shared::data::index_file::VersionIndex;
use shared::data::version_meta::VersionMeta;
use shared::data::version_meta_group::VersionMetaGroup;

use crate::common::archive_tester::ArchiveTester;
use crate::common::tar_reader::TarReader;
use crate::common::tar_writer::TarWriter;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::web::webstate::WebState;

#[derive(Deserialize)]
pub struct RequestBody {
    /// 新包的版本号
    label: String,

    /// 新包的更新记录
    change_logs: String,
}

/// 打包新版本
pub async fn api_pack(State(state): State<WebState>, headers: HeaderMap, Json(payload): Json<RequestBody>) -> Response {
    let wait = headers.get("wait").is_some();

    state.clone().te.lock().await
        .try_schedule(wait, state.clone(), move || do_check(payload, state)).await
}

fn do_check(payload: RequestBody, state: WebState) -> u8 {
    let version_label = payload.label;
    let change_logs = payload.change_logs;

    let mut console = state.console.blocking_lock();
    
    let mut index_file = IndexFile::load_from_file(&state.app_path.index_file);

    if index_file.contains(&version_label) {
        console.log_error(format!("版本号已经存在: {}", version_label));
        return 1;
    }

    // 1. 读取所有历史版本，并推演出上个版本的文件状态，用于和工作空间目录对比生成文件差异
    // 读取现有更新包，并复现在history上
    console.log_debug("正在读取数据");

    let mut history = HistoryFile::new_dir("workspace_root", Weak::new());

    for v in &index_file {
        let mut reader = TarReader::new(state.app_path.public_dir.join(&v.filename));
        let meta_group = reader.read_metadata_group(v.offset, v.len);

        for meta in &meta_group {
            history.replay_operations(&meta);
        }
    }

    // 对比文件
    console.log_debug("正在扫描文件更改");

    let exclude_rules = &state.config.core.exclude_rules;
    let disk_file = DiskFile::new(state.app_path.workspace_dir.clone(), Weak::new());
    let diff = Diff::diff(&disk_file, &history, Some(exclude_rules));

    if !diff.has_diff() {
        console.log_error("目前工作目录还没有任何文件修改");
        return 1;
    }

    console.log_info(format!("{:#?}", diff));

    // 2. 将所有“覆盖的文件”的数据和元数据写入到更新包中，同时更新元数据中每个文件的偏移值
    // 创建新的更新包，将所有文件修改写进去
    std::fs::create_dir_all(&state.app_path.public_dir).unwrap();
    let version_filename = format!("{}.tar", version_label);
    let version_file = state.app_path.public_dir.join(&version_filename);
    let mut writer = TarWriter::new(&version_file);

    // 写入每个更新的文件数据
    let mut vec = Vec::<&DiskFile>::new();
    
    for f in &diff.added_files {
        vec.push(f);
    }

    for f in &diff.modified_files {
        vec.push(f);
    }

    let mut counter = 1;
    for f in &vec {
        console.log_debug(format!("打包({}/{}) {}", counter, vec.len(), f.path().deref()));
        counter += 1;

        let path = f.path().to_owned();
        let disk_file = state.app_path.workspace_dir.join(&path);
        let open = std::fs::File::options().read(true).open(disk_file).unwrap();

        // 提供的len必须和读取到的长度严格相等
        let meta = open.metadata().unwrap();
        assert_eq!(meta.len(), f.len());

        writer.add_file(open, f.len(), &path, &version_label);
    }

    // 写入元数据
    console.log_debug("写入元数据");

    // 读取写好的更新记录
    let meta = VersionMeta::new(version_label.clone(), change_logs, diff.to_file_changes());
    let meta_group = VersionMetaGroup::with_one(meta);
    let meta_info = writer.finish(meta_group);

    // 3. 更新索引文件
    index_file.add(VersionIndex {
        label: version_label.to_owned(),
        filename: version_filename,
        offset: meta_info.offset,
        len: meta_info.length,
        hash: "no hash".to_owned(),
    });

    // 进行解压测试
    console.log_debug("正在测试");

    let mut tester = ArchiveTester::new();
    for v in &index_file {
        tester.feed(state.app_path.public_dir.join(&v.filename), v.offset, v.len);
    }
    tester.finish(|e| console.log_debug(format!("{}/{} 正在测试 {} 的 {} ({}+{})", e.index, e.total, e.label, e.path, e.offset, e.len))).unwrap();

    console.log_info("测试通过，打包完成！");
    
    index_file.save(&state.app_path.index_file);

    // // 生成上传脚本
    // let context = TemplateContext {
    //     upload_files: vec![version_file.strip_prefix(&ctx.working_dir).unwrap().to_str().unwrap().to_owned()],
    //     delete_files: Vec::new(),
    // };

    // generate_upload_script(context, ctx, &version_label);

    0
}