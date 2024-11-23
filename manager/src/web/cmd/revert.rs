use std::fs::FileTimes;
use std::ops::Deref;
use std::rc::Weak;

use axum::extract::State;
use axum::response::Response;
use shared::data::index_file::IndexFile;

use crate::common::tar_reader::TarReader;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::web::webstate::WebState;

/// 恢复工作空间目录到未修改的时候
/// 
/// 有时可能修改了工作空间目录下的文件，但是觉得不满意，想要退回未修改之前，那么可以使用revert命令
pub async fn api_revert(State(state): State<WebState>) -> Response {
    state.clone().te.lock().await
        .try_schedule(move || do_revert(state)).await
}

pub fn do_revert(state: WebState) {
    let config = state.config;
    let mut console = state.console.blocking_lock();

    let index_file = IndexFile::load_from_file(&config.index_file);

    // 读取现有更新包，并复现在history上
    console.log("正在读取数据");

    let mut history = HistoryFile::new_empty();

    for v in &index_file {
        let mut reader = TarReader::new(config.public_dir.join(&v.filename));
        let meta_group = reader.read_metadata_group(v.offset, v.len);

        for meta in meta_group {
            history.replay_operations(&meta);
        }
    }

    // 对比文件
    console.log("正在扫描文件更改");

    let exclude_rules = &config.config.blocking_lock().core.exclude_rules;
    let disk_file = DiskFile::new(config.workspace_dir.clone(), Weak::new());
    let diff = Diff::diff(&history, &disk_file, Some(exclude_rules));
    drop(disk_file);

    // 输出文件差异
    // if is_running_under_cargo() {
    //     // console.log("{:#?}", diff);
    //     // console.log("{}", diff);
    // }

    // 退回
    console.log("正在退回文件修改");

    for mk in diff.added_folders {
        let dir = config.workspace_dir.join(mk.path().deref());

        if let Err(e) = std::fs::create_dir_all(dir) {
            panic!("{}: {:?}", mk.path().deref(), e);
        }
    }

    for mv in diff.renamed_files {
        let src = mv.0.disk_file();
        let dst = config.workspace_dir.join(mv.1.path().deref());

        if let Err(e) = std::fs::rename(src, dst) {
            panic!("{} => {}: {:?}", mv.0.path().deref(), mv.1.path().deref(), e);
        }
    }

    for rm in diff.missing_files {
        let file = rm.disk_file();

        if let Err(e) = std::fs::remove_file(file) {
            panic!("{}({}): {:?}", rm.path().deref(), file.to_str().unwrap(), e);
        }
    }

    for rm in diff.missing_folders {
        let dir = rm.disk_file();
        
        if let Err(e) = std::fs::remove_dir(dir) {
            panic!("{}: {:?}", rm.path().deref(), e);
        }
    }

    let mut vec = Vec::<&HistoryFile>::new();

    for f in &diff.added_files {
        vec.push(&f);
    }

    for f in &diff.modified_files {
        vec.push(&f);
    }

    for up in vec {
        let file = config.workspace_dir.join(up.path().deref());

        let loc = up.file_location();
        let meta = index_file.find(&loc.version).unwrap();
        
        let mut reader = TarReader::new(config.public_dir.join(&meta.filename));

        let open = std::fs::File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file);

        let mut open = match open {
            Ok(open) => open,
            Err(e) => panic!("{}: {}", up.path().deref(), e.to_string()),
        };

        let mut src = reader.open_file(loc.offset, loc.length as u64);

        std::io::copy(&mut src, &mut open).unwrap();

        open.set_times(FileTimes::new().set_modified(up.modified())).unwrap();
    }

    console.log("工作空间目录已经退回到未修改之前");
}