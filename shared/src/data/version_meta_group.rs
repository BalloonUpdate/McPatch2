//! 版本元数据组
//! 
//! 版本元数据组代表一个元数据列表。在存储时会序列化成一个Json列表
//! 
//! 元数据列表因为是列表，可以存储多个元数据。但实际中只有合并包里才会存储多个版本，普通包中只存一个
//! 
//! ```json
//! [
//!     {
//!         "label": "1.0", // 版本号
//!         "logs": "这是这个版本的更新记录文字示例", // 这个版本的更新日志
//!         "changes": [] // 记录所有文件修改操作
//!     },
//!     {
//!         "label": "1.2", // 版本号
//!         "logs": "这是1.0版本的更新记录文字示例", // 这个版本的更新日志
//!         "changes": [] // 记录所有文件修改操作
//!     },
//!     {
//!         "label": "1.3", // 版本号
//!         "logs": "这是1.1版本的更新记录文字示例", // 这个版本的更新日志
//!         "changes": [] // 记录所有文件修改操作
//!     }
//! ]
//! ```

use json::JsonValue;

use crate::data::version_meta::VersionMeta;
use crate::utility::vec_ext::VecRemoveIf;

/// 代表一组版本元数据，每个更新包tar文件都能容纳多个版本的元数据，也叫一组
pub struct VersionMetaGroup(pub Vec<VersionMeta>);

impl VersionMetaGroup {
    /// 创建一个组空的元数据
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// 创建单个元数据组
    pub fn with_one(meta: VersionMeta) -> Self {
        Self([meta].into())
    }

    /// 从json字符串进行解析
    pub fn parse(meta: &str) -> Self {
        let root = json::parse(meta).unwrap();

        VersionMetaGroup(root.members().map(|e| VersionMeta::load(e)).collect())
    }

    /// 将元数据组序列化成json字符串
    pub fn serialize(&self) -> String {
        let mut obj = JsonValue::new_array();

        for v in &self.0 {
            obj.push(v.serialize()).unwrap();
        }

        obj.pretty(4)
    }

    // /// 添加一个元数据组
    // pub fn add_group(&mut self, group: VersionMetaGroup) {
    //     for meta in group.0 {
    //         if self.contains_meta(&meta.label) {
    //             continue;
    //         }

    //         self.add_meta(meta);
    //     }
    // }

    /// 添加单个元数据
    pub fn add_meta(&mut self, meta: VersionMeta) {
        assert!(!self.contains_meta(&meta.label));

        self.0.push(meta);
    }

    /// 删除一个元数据
    pub fn remove_meta(&mut self, label: &str) -> bool {
        self.0.remove_if(|e| e.label == label)
    }

    /// 检查是否包括一个元数据
    pub fn contains_meta(&self, label: &str) -> bool {
        self.0.iter().any(|e| e.label == label)
    }

    /// 查找一个元数据
    pub fn find_meta(&self, label: &str) -> Option<&VersionMeta> {
        self.0.iter().find(|e| e.label == label)
    }
}

impl IntoIterator for VersionMetaGroup {
    type Item = VersionMeta;

    type IntoIter = std::vec::IntoIter<VersionMeta>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a VersionMetaGroup {
    type Item = &'a VersionMeta;

    type IntoIter = std::slice::Iter<'a, VersionMeta>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a> IntoIterator for &'a mut VersionMetaGroup {
    type Item = &'a mut VersionMeta;

    type IntoIter = std::slice::IterMut<'a, VersionMeta>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}