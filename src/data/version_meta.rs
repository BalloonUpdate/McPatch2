use std::collections::LinkedList;
use std::ops::Add;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use json::JsonValue;

pub enum FileChange {
    CreateFolder { path: String },
    UpdateFile { path: String, hash: String, len: u64, modified: SystemTime, offset: u64 },
    DeleteFolder { path: String },
    DeleteFile { path: String },
    MoveFile { from: String, to: String },
}

pub struct VersionMeta {
    pub logs: String,
    pub changes: LinkedList<FileChange>,
}

impl VersionMeta {
    pub fn new(logs: String, changes: LinkedList<FileChange>) -> Self {
        Self { logs, changes }
    }

    pub fn load(str: &str) -> Self {
        let root = json::parse(str).unwrap();

        Self {
            logs: root["logs"].as_str().unwrap().to_owned(), 
            changes: root["changes"].members()
                .map(|e| Self::parse_change(e))
                .collect::<LinkedList<_>>()
        }
    }

    pub fn save(&self) -> String {
        let mut root = JsonValue::new_object();
        let mut changes = JsonValue::new_array();
        
        for change in &self.changes {
            changes.push(Self::serialize_change(change)).unwrap();
        }
        
        root.insert("logs", self.logs.to_owned()).unwrap();
        root.insert("changes", changes).unwrap();
        root.pretty(4)
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