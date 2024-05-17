//! 恢复工作空间目录到未修改的时候
//! 
//! 有时可能修改了工作空间目录下的文件，但是觉得不满意，想要退回未修改之前，那么可以使用revert命令

use std::ops::Deref;
use std::rc::Weak;

use mcpatch_shared::data::index_file::IndexFile;

use crate::common::tar_reader::TarReader;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::AppContext;

pub fn do_revert(ctx: &AppContext) -> i32 {
    let index_file = IndexFile::load_from_file(&ctx.index_file);

    // 读取现有更新包，并复现在history上
    println!("正在读取数据");

    let mut history = HistoryFile::new_empty();

    for v in &index_file {
        let mut reader = TarReader::new(ctx.public_dir.join(&v.filename));
        let meta_group = reader.read_metadata_group(v.offset, v.len);

        for meta in meta_group {
            history.replay_operations(&meta, v.into());
        }
    }

    // 对比文件
    println!("正在扫描文件更改");

    let disk_file = DiskFile::new(ctx.workspace_dir.clone(), Weak::new());
    let diff = Diff::diff(&history, &disk_file, Some(&ctx.config.filter_rules));

    // 输出文件差异
    println!("{:#?}", diff);
    println!("{}", diff);

    // 退回
    println!("正在退回文件修改");

    for mk in diff.created_folders {
        let dir = ctx.workspace_dir.join(mk.path().deref());

        std::fs::create_dir_all(dir).unwrap();
    }

    for mv in diff.renamed_files {
        let src = mv.0.disk_file();
        let dst = ctx.workspace_dir.join(mv.1.path().deref());

        println!("d  {}  --->  {}", src.to_str().unwrap(), dst.to_str().unwrap());

        std::fs::rename(src, dst).unwrap();
    }

    for rm in diff.deleted_files {
        let file = rm.disk_file();

        std::fs::remove_file(file).unwrap();
    }

    for rm in diff.deleted_folders {
        let dir = rm.disk_file();
        
        std::fs::remove_dir(dir).unwrap();
    }

    for up in diff.updated_files {
        let file = ctx.workspace_dir.join(up.path().deref());

        let loc = up.file_location();
        let meta = index_file.find("").unwrap();
        // let meta = index_file.find(&loc.version.unwrap()).unwrap();
        
        let mut reader = TarReader::new(ctx.public_dir.join(&meta.filename));

        let mut open = std::fs::File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(file)
            .unwrap();

        let mut src = reader.open_file(loc.offset, loc.length as u64);

        std::io::copy(&mut src, &mut open).unwrap();
    }

    println!("工作空间目录已经退回到未修改之前");

    0
}