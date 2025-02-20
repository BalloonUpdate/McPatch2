use std::fs::FileTimes;
use std::ops::Deref;
use std::rc::Weak;

use shared::data::index_file::IndexFile;

use crate::app_path::AppPath;
use crate::config::Config;
use crate::core::tar_reader::TarReader;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::web::log::Console;


pub fn task_revert(apppath: &AppPath, config: &Config, console: &Console) -> u8 {
    let index_file = IndexFile::load_from_file(&apppath.index_file);

    // 读取现有更新包，并复现在history上
    console.log_debug("正在读取数据");

    let mut history = HistoryFile::new_empty();

    for v in &index_file {
        let mut reader = TarReader::new(apppath.public_dir.join(&v.filename));
        let meta_group = reader.read_metadata_group(v.offset, v.len);

        for meta in meta_group {
            history.replay_operations(&meta);
        }
    }

    // 对比文件
    console.log_debug("正在扫描文件更改");

    let exclude_rules = &config.core.exclude_rules;
    let disk_file = DiskFile::new(apppath.workspace_dir.clone(), Weak::new());
    let diff = Diff::diff(&history, &disk_file, Some(exclude_rules));
    drop(disk_file);

    // 输出文件差异
    // if is_running_under_cargo() {
    //     // console.log("{:#?}", diff);
    //     // console.log("{}", diff);
    // }

    // 退回
    console.log_debug("正在退回文件修改");

    for mk in diff.added_folders {
        let dir = apppath.workspace_dir.join(mk.path().deref());

        if let Err(e) = std::fs::create_dir_all(dir) {
            panic!("{}: {:?}", mk.path().deref(), e);
        }
    }

    for mv in diff.renamed_files {
        let src = mv.0.disk_file();
        let dst = apppath.workspace_dir.join(mv.1.path().deref());

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
        let file = apppath.workspace_dir.join(up.path().deref());

        let loc = up.file_location();
        let meta = index_file.find(&loc.version).unwrap();
        
        let mut reader = TarReader::new(apppath.public_dir.join(&meta.filename));

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

    console.log_info("工作空间目录已经退回到未修改之前");

    0
}