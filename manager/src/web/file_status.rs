use std::rc::Weak;

use crate::app_path::AppPath;
use crate::core::data::index_file::IndexFile;
use crate::config::Config;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;

pub struct FileStatus {
    pub app_path: AppPath,
    pub config: Config,
    pub status: Option<Status>,
}

impl FileStatus {
    pub fn new(app_path: AppPath, config: Config) -> Self {
        Self { app_path, config, status: None }
    }

    /// 清空缓存，下次会进行重建
    pub fn invalidate(&mut self) {
        self.status = None;
    }

    /// 获取一个文件的修改状态
    pub async fn get_file_status(&mut self, path: &str) -> SingleFileStatus {
        let status = self.refresh().await;

        let path = &path.to_string();

        // println!("> {}", path);

        if status.added_folders.contains(path) {
            // println!("1 {}", join_string(status.added_folders.iter().map(|e| e.to_owned()), "\n"));
            return SingleFileStatus::Added;
        }

        if status.added_files.contains(path) {
            // println!("2");
            return SingleFileStatus::Added;
        }

        if status.modified_files.contains(path) {
            // println!("3");
            return SingleFileStatus::Modified;
        }

        if status.missing_folders.contains(path) {
            // println!("4");
            return SingleFileStatus::Missing;
        }

        if status.missing_files.contains(path) {
            // println!("5");
            return SingleFileStatus::Missing;
        }

        if status.gone_files.contains(path) {
            // println!("6");
            return SingleFileStatus::Gone;
        }

        if status.come_files.contains(path) {
            // println!("7");
            return SingleFileStatus::Come;
        }

        // 如果目录下有文件有变动，也要视为修改状态
        if status.added_folders.iter().any(|e| e.starts_with(path)) {
            // println!("a");
            return SingleFileStatus::Modified;
        }
        if status.added_files.iter().any(|e| e.starts_with(path)) {
            // println!("b");
            return SingleFileStatus::Modified;
        }
        if status.modified_files.iter().any(|e| e.starts_with(path)) {
            // println!("c");
            return SingleFileStatus::Modified;
        }
        if status.missing_folders.iter().any(|e| e.starts_with(path)) {
            // println!("d");
            return SingleFileStatus::Modified;
        }
        if status.missing_files.iter().any(|e| e.starts_with(path)) {
            // println!("e");
            return SingleFileStatus::Modified;
        }
        if status.gone_files.iter().any(|e| e.starts_with(path)) {
            // println!("f");
            return SingleFileStatus::Modified;
        }
        if status.come_files.iter().any(|e| e.starts_with(path)) {
            // println!("g");
            return SingleFileStatus::Modified;
        }

        // println!("8");
        return SingleFileStatus::Keep;
    }

    /// 尝试重新生成文件状态缓存
    async fn refresh(&mut self) -> &Status {
        if self.status.is_none() {
            println!("rebuild cache");

            let app_path = &self.app_path;

            // 读取现有更新包，并复现在history上
            let index_file = IndexFile::load_from_file(&app_path.index_file);

            let mut history = HistoryFile::new_empty();

            for (_index, meta) in index_file.read_all_metas(&app_path.public_dir) {
                history.replay_operations(&meta);
            }

            // 对比文件
            let exclude_rules = &self.config.core.exclude_rules;
            let disk_file = DiskFile::new(app_path.workspace_dir.clone(), Weak::new());
            let diff = Diff::diff(&disk_file, &history, Some(&exclude_rules));

            let mut status = Status::default();
            
            for f in diff.added_folders {
                status.added_folders.push(f.path().to_owned());
            }

            for f in diff.added_files {
                status.added_files.push(f.path().to_owned());
            }

            for f in diff.modified_files {
                status.modified_files.push(f.path().to_owned());
            }

            for f in diff.missing_folders {
                status.missing_folders.push(f.path().to_owned());
            }

            for f in diff.missing_files {
                status.missing_files.push(f.path().to_owned());
            }

            for f in diff.renamed_files {
                status.gone_files.push(f.0.path().to_owned());
                status.come_files.push(f.1.path().to_owned());
            }

            self.status = Some(status);
        }

        return &self.status.as_ref().unwrap();
    }
}

pub enum SingleFileStatus {
    /// 文件无变更
    Keep,

    /// 新增的文件或者目录
    Added,

    /// 修改的文件或者目录
    Modified,

    /// 删除的文件或者目录
    Missing,

    /// 被移动走的的文件或者目录
    Gone,

    /// 被移动过来的文件或者目录
    Come,
}

/// 保存计算出来的文件状态缓存
#[derive(Default)]
pub struct Status {
    pub added_folders: Vec<String>,
    pub added_files: Vec<String>,
    pub modified_files: Vec<String>,
    pub missing_folders: Vec<String>,
    pub missing_files: Vec<String>,
    pub gone_files: Vec<String>,
    pub come_files: Vec<String>,
}
