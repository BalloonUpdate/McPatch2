use std::path::Path;

use json::JsonValue;

/// 代表一个版本的索引信息
#[derive(Clone)]
pub struct VersionIndex {
    /// 版本号
    pub label: String,

    /// 版本的数据存在哪个文件里
    pub filename: String,

    /// 元数据组的偏移值
    pub offset: u64,

    /// 元数据组的长度
    pub len: u32,

    /// 元数据组整个字符串的哈希值
    pub hash: String,
}

/// 代表一个索引文件
pub struct IndexFile {
    versions: Vec<VersionIndex>
}

impl IndexFile {
    /// 从`index_file`加载索引文件
    pub fn load(index_file: &Path) -> Self {
        let content = std::fs::read_to_string(index_file)
            .unwrap_or_else(|_| "[]".to_owned());
        
        let root = json::parse(&content).unwrap();
        let mut versions = Vec::<VersionIndex>::new();

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

    /// 将索引数据写到`index_file`文件里
    pub fn save(&self, index_file: &Path) {
        let mut root = JsonValue::new_array();

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

    /// 添加一个新版本
    pub fn add_index(&mut self, version: VersionIndex) {
        self.versions.push(version);
    }

    /// 检查是否包含指定的版本号
    pub fn contains_label(&self, label: &str) -> bool {
        self.versions.iter().any(|e| e.label == label)
    }

    /// 查找一个版本的索引数据
    pub fn find_version_index(&self, label: &str) -> Option<&VersionIndex> {
        self.versions.iter().find(|e| e.label == label)
    }

    /// 查找一个版本的可变索引数据
    pub fn find_version_index_mut(&mut self, label: &str) -> Option<&mut VersionIndex> {
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