use std::collections::LinkedList;
use std::ops::Add;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use json::JsonValue;

/// 代表单个文件操作
pub enum FileChange {
    CreateFolder { path: String },
    UpdateFile { path: String, hash: String, len: u64, modified: SystemTime, offset: u64 },
    DeleteFolder { path: String },
    DeleteFile { path: String },
    MoveFile { from: String, to: String },
}

/// 代表一个版本的元数据
pub struct VersionMeta {
    /// 版本号或者标签
    pub label: String,

    /// 版本的更新日志
    pub logs: String,

    /// 文件变动情况
    pub changes: LinkedList<FileChange>,
}

impl VersionMeta {
    /// 创建一个新的版本元数据
    pub fn new(label: String, logs: String, changes: LinkedList<FileChange>) -> Self {
        Self { label, logs, changes }
    }

    /// 加载一个现有的版本元数据
    pub fn load(obj: &JsonValue) -> Self {
        Self {
            label: obj["label"].as_str().unwrap().to_owned(),
            logs: obj["logs"].as_str().unwrap().to_owned(), 
            changes: obj["changes"].members()
                .map(|e| Self::parse_change(e))
                .collect::<LinkedList<_>>()
        }
    }

    /// 将版本元数据保存成JsonObject格式
    pub fn serialize(&self) -> JsonValue {
        let mut obj = JsonValue::new_object();
        let mut changes = JsonValue::new_array();
        
        for change in &self.changes {
            changes.push(Self::serialize_change(change)).unwrap();
        }
        
        obj.insert("label", self.label.clone()).unwrap();
        obj.insert("logs", self.logs.clone()).unwrap();
        obj.insert("changes", changes).unwrap();
        obj
    }

    fn parse_change(v: &JsonValue) -> FileChange {
        match v["operation"].as_str().unwrap() {
            "create-directory" => {
                FileChange::CreateFolder {
                    path: v["path"].as_str().unwrap().to_owned()
                }
            },
            "update-file" => {
                FileChange::UpdateFile {
                    path: v["path"].as_str().unwrap().to_owned(), 
                    hash: v["hash"].as_str().unwrap().to_owned(), 
                    len: v["len"].as_u64().unwrap(), 
                    modified: UNIX_EPOCH.add(Duration::from_secs(v["modified"].as_u64().unwrap())), 
                    offset: v["offset"].as_u64().unwrap(),
                }
            },
            "delete-directory" => {
                FileChange::DeleteFolder {
                    path: v["path"].as_str().unwrap().to_owned(), 
                }
            },
            "delete-file" => {
                FileChange::DeleteFile {
                    path: v["path"].as_str().unwrap().to_owned(), 
                }
            },
            "move-file" => {
                FileChange::MoveFile {
                    from: v["from"].as_str().unwrap().to_owned(), 
                    to: v["to"].as_str().unwrap().to_owned(),
                }
            },
            _ => panic!("unknown operation: {}", v["operation"].as_str().unwrap())
        }
    }

    fn serialize_change(change: &FileChange) -> JsonValue {
        let mut obj = JsonValue::new_object();

        match change {
            FileChange::CreateFolder { path } => {
                obj.insert("operation", "create-directory").unwrap();
                obj.insert("path", path.to_owned()).unwrap();
            },
            FileChange::UpdateFile { path, hash, len, modified, offset } => {
                obj.insert("operation", "update-file").unwrap();
                obj.insert("path", path.to_owned()).unwrap();
                obj.insert("hash", hash.to_owned()).unwrap();
                obj.insert("len", len.to_owned()).unwrap();
                obj.insert("modified", modified.duration_since(UNIX_EPOCH).unwrap().as_secs()).unwrap();
                obj.insert("offset", offset.to_owned()).unwrap();
            },
            FileChange::DeleteFolder { path } => {
                obj.insert("operation", "delete-directory").unwrap();
                obj.insert("path", path.to_owned()).unwrap();
            },
            FileChange::DeleteFile { path } => {
                obj.insert("operation", "delete-file").unwrap();
                obj.insert("path", path.to_owned()).unwrap();
            },
            FileChange::MoveFile { from, to } => {
                obj.insert("operation", "move-file").unwrap();
                obj.insert("from", from.to_owned()).unwrap();
                obj.insert("to", to.to_owned()).unwrap();
            },
        }

        obj
    }
}