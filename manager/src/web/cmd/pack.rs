use std::collections::HashMap;
use std::io::ErrorKind;
use std::ops::Deref;
use std::rc::Weak;

use axum::body::Body;
use axum::extract::Query;
use axum::extract::State;
use axum::response::Response;
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

/// 打包新版本
pub async fn api_pack(Query(params): Query<HashMap<String, String>>, State(state): State<WebState>) -> Response {
    let label = match params.get("label") {
        Some(ok) => ok.to_owned(),
        None => return Response::builder()
            .status(500)
            .body(Body::new("parameter 'label' is missing.".to_string()))
            .unwrap(),
    };

    state.clone().te.lock().await
        .try_schedule(move || do_check(label, state)).await
}

fn do_check(version_label: String, state: WebState) {
    let ctx = state.app_context;
    let mut console = state.console.blocking_lock();
    
    let mut index_file = IndexFile::load_from_file(&ctx.index_file);

    if index_file.contains(&version_label) {
        console.log(format!("版本号已经存在: {}", version_label));
        return;
    }

    // 1. 读取所有历史版本，并推演出上个版本的文件状态，用于和工作空间目录对比生成文件差异
    // 读取现有更新包，并复现在history上
    console.log("正在读取数据");

    let mut history = HistoryFile::new_dir("workspace_root", Weak::new());

    for v in &index_file {
        let mut reader = TarReader::new(ctx.public_dir.join(&v.filename));
        let meta_group = reader.read_metadata_group(v.offset, v.len);

        for meta in &meta_group {
            history.replay_operations(&meta);
        }
    }

    // 对比文件
    console.log("正在扫描文件更改");

    let disk_file = DiskFile::new(ctx.workspace_dir.clone(), Weak::new());
    let diff = Diff::diff(&disk_file, &history, Some(&ctx.config.exclude_rules));

    if !diff.has_diff() {
        console.log("目前工作目录还没有任何文件修改");
        return;
    }

    console.log(format!("{:#?}", diff));

    // 2. 将所有“覆盖的文件”的数据和元数据写入到更新包中，同时更新元数据中每个文件的偏移值
    // 创建新的更新包，将所有文件修改写进去
    std::fs::create_dir_all(&ctx.public_dir).unwrap();
    let version_filename = format!("{}.tar", version_label);
    let version_file = ctx.public_dir.join(&version_filename);
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
        console.log(format!("打包({}/{}) {}", counter, vec.len(), f.path().deref()));
        counter += 1;

        let path = f.path().to_owned();
        let disk_file = ctx.workspace_dir.join(&path);
        let open = std::fs::File::options().read(true).open(disk_file).unwrap();

        // 提供的len必须和读取到的长度严格相等
        let meta = open.metadata().unwrap();
        assert_eq!(meta.len(), f.len());

        writer.add_file(open, f.len(), &path, &version_label);
    }

    // 写入元数据
    console.log("写入元数据");

    // 读取写好的更新记录

    // 创建空的logs.txt文件，以提醒用户可以在这里写更新记录
    let logs_file = ctx.working_dir.join("logs.txt");
    if let Err(e) = std::fs::metadata(&logs_file) {
        if e.kind() == ErrorKind::NotFound {
            std::fs::write(&logs_file, &[]).unwrap();
        }
    }
    let logs = match std::fs::read_to_string(&logs_file) {
        Ok(text) => text,
        Err(_) => "没有更新记录".to_owned(),
    };

    let meta = VersionMeta::new(version_label.clone(), logs, diff.to_file_changes());
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
    console.log("正在测试");

    let mut tester = ArchiveTester::new();
    for v in &index_file {
        tester.feed(ctx.public_dir.join(&v.filename), v.offset, v.len);
    }
    tester.finish(|e| console.log(format!("{}/{} 正在测试 {} 的 {} ({}+{})", e.index, e.total, e.label, e.path, e.offset, e.len))).unwrap();

    console.log("测试通过，打包完成！");
    
    index_file.save(&ctx.index_file);

    // // 生成上传脚本
    // let context = TemplateContext {
    //     upload_files: vec![version_file.strip_prefix(&ctx.working_dir).unwrap().to_str().unwrap().to_owned()],
    //     delete_files: Vec::new(),
    // };

    // generate_upload_script(context, ctx, &version_label);
}