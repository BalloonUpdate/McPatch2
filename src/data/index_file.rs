use std::path::Path;

use json::JsonValue;

/// 代表一个版本的索引信息
#[derive(Clone)]
pub struct VersionIndex {
    pub label: String,
    pub filename: String,
    pub offset: u64,
    pub len: u32,
    pub hash: String,
}

pub struct IndexFile {
    versions: Vec<VersionIndex>
}

impl IndexFile {
    pub fn load(index_file: &Path) -> Self {
        let content = match std::fs::read_to_string(index_file) {
            Ok(content) => content,
            Err(_) => "[]".to_owned(),
        };

        let mut versions = Vec::<VersionIndex>::new();

        let root = json::parse(&content).unwrap();

        for v in root.members() {
            let label = v["label"].as_str().unwrap().to_owned();
            let filename = v["filename"].as_str().unwrap().to_owned();
            let offset = v["offset"].as_u64().unwrap();
            let len = v["length"].as_u32().unwrap();
            let hash = v["hash"].as_str().unwrap().to_owned();

            versions.push(VersionIndex { label, filename, len, offset, hash })
        }

        Self { versions }
    }

    pub fn save(&self, index_file: &Path) {
        let mut root = JsonValue::new_array();

        // let dddd = &self.versions.into_iter();

        for v in &self.versions {
            let mut obj = JsonValue::new_object();

            obj.insert("label", v.label.to_owned()).unwrap();
            obj.insert("filename", v.filename.to_owned()).unwrap();
            obj.insert("offset", v.offset).unwrap();
            obj.insert("length", v.len).unwrap();
            obj.insert("hash", v.hash.to_owned()).unwrap();
            
            root.push(obj).unwrap();
        }

        std::fs::write(index_file, root.pretty(4)).unwrap()
    }

    pub fn add_index(&mut self, version: VersionIndex) {
        self.versions.push(version);
    }

    pub fn contains_label(&self, label: &str) -> bool {
        self.versions.iter().any(|e| e.label == label)
    }

    pub fn get_index(&self, label: &str) -> Option<&VersionIndex> {
        self.versions.iter().find(|e| e.label == label)
    }

    pub fn get_index_mut(&mut self, label: &str) -> Option<&mut VersionIndex> {
        self.versions.iter_mut().find(|e| e.label == label)
    }
}

impl<'a> IntoIterator for &'a IndexFile {
    type Item = &'a VersionIndex;

    type IntoIter = std::slice::Iter<'a, VersionIndex>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.versions).into_iter()
    }
}

impl IntoIterator for IndexFile {
    type Item = VersionIndex;

    type IntoIter = std::vec::IntoIter<VersionIndex>;

    fn into_iter(self) -> Self::IntoIter {
        self.versions.into_iter()
    }
}