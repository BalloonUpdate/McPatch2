use json::JsonValue;

use crate::data::version_meta::VersionMeta;

/// 代表一组版本元数据，每个更新包tar文件都能容纳多个版本的元数据，也叫一组
pub struct VersionMetaGroup(Vec<VersionMeta>);

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

    /// 添加一个元数据组
    pub fn add_group(&mut self, group: VersionMetaGroup) {
        for meta in group.0 {
            self.add_meta(meta);
        }
    }

    /// 添加单个元数据
    pub fn add_meta(&mut self, meta: VersionMeta) {
        self.0.push(meta);
    }

    /// 查找一个元数据
    pub fn find_meta(&self, label: &str) -> Option<&VersionMeta> {
        self.0.iter().find(|e| e.label == label)
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