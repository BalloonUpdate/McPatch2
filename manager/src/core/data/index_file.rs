//! 版本索引

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Index;
use std::path::Path;

use json::JsonValue;

use crate::core::data::version_meta::VersionMeta;
use crate::core::data::version_meta_group::VersionMetaGroup;
use crate::core::tar_reader::TarReader;

/// 代表一个版本的索引信息
/// 
/// 保存时会被序列化成一个Json对象
/// 
/// ```json
/// {
///     "label": "1.2",
///     "file": "1.2.tar",
///     "offset": 7A9C,
///     "length": 1000,
///     "hash": "23B87EA52C893"
/// }
/// ```
#[derive(Clone)]
pub struct VersionIndex {
    /// 版本号
    pub label: String,

    /// 版本的数据存在哪个文件里
    pub filename: String,

    /// 元数据组的偏移值
    pub offset: u64,

    /// 元数据组的长度
    pub len: u64,

    /// 整个tar包文件的校验
    pub hash: String,
}

/// 代表一个索引文件
pub struct IndexFile {
    versions: Vec<VersionIndex>
}

impl IndexFile {
    /// 创建一个IndexFile
    pub fn new() -> Self {
        Self { versions: Vec::new() }
    }

    /// 从文件加载索引文件
    pub fn load_from_file(index_file: &Path) -> Self {
        let content = std::fs::read_to_string(index_file)
            .unwrap_or_else(|_| "[]".to_owned());
        
        Self::load_from_json(&content)
    }

    /// 从Json字符串加载
    pub fn load_from_json(json: &str) -> Self {
        let root = json::parse(json).unwrap();
        let mut versions = Vec::<VersionIndex>::new();

        for v in root.members() {
            let label = v["label"].as_str().unwrap().to_owned();
            let filename = v["filename"].as_str().unwrap().to_owned();
            let offset = v["offset"].as_u64().unwrap();
            let len = v["length"].as_u64().unwrap();
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
    pub fn add(&mut self, version: VersionIndex) {
        self.versions.push(version);
    }

    /// 检查是否包含指定的版本号
    pub fn contains(&self, label: &str) -> bool {
        self.versions.iter().any(|e| e.label == label)
    }

    /// 查找一个版本的索引数据
    pub fn find(&self, label: &str) -> Option<&VersionIndex> {
        self.versions.iter().find(|e| e.label == label)
    }

    /// 查找一个版本的可变索引数据
    pub fn find_mut(&mut self, label: &str) -> Option<&mut VersionIndex> {
        self.versions.iter_mut().find(|e| e.label == label)
    }

    /// 版本的数量
    pub fn len(&self) -> usize {
        self.versions.len()
    }

    /// 读取所有的meta数据
    /// 收集所有需要读取的元数据信息，同时进行去重，避免一个文件的相同部分被读取多遍，虽然读不满，但是解析很慢
    pub fn read_all_metas(&self, public_dir: &Path) -> Vec::<(VersionIndex, VersionMeta)> {
        let relevant_files = self.versions
            .iter()
            .map(|e| format!("{}|{}|{}", e.filename, e.offset, e.len))
            .collect::<HashSet<String>>();
        
        let mut reading_cache = HashMap::<String, VersionMetaGroup>::new();
        
        // 读取所有元数据
        for file in relevant_files {
            let mut split = file.split("|");
            
            let filename = split.next().unwrap();
            let offset = u64::from_str_radix(split.next().unwrap(), 10).unwrap();
            let len = u64::from_str_radix(split.next().unwrap(), 10).unwrap();
            
            let mut reader = TarReader::new(public_dir.join(&filename));

            reading_cache.insert(file, reader.read_metadata_group(offset, len));
        }

        // 再根据索引文件里的内容进行返回
        let mut metas = Vec::<(VersionIndex, VersionMeta)>::new();
        
        for v in &self.versions {
            let cache_key = format!("{}|{}|{}", v.filename, v.offset, v.len);
            let group = reading_cache.get(&cache_key).unwrap();
            let meta = group.find_meta(&v.label).unwrap();
            
            metas.push((v.clone(), meta.to_owned()));
        }

        metas
    }
}

impl Index<usize> for IndexFile {
    type Output = VersionIndex;

    fn index(&self, index: usize) -> &Self::Output {
        &self.versions[index]
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