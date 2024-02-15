use json::JsonValue;

use crate::data::version_meta::VersionMeta;

/// 代表单个tar文件中的所有版本的元数据，每个tar文件都能容纳多个版本的数据
pub struct VersionMetaGroup(Vec<VersionMeta>);

impl VersionMetaGroup {
    /// 创建一个组空的元数据
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// 从json字符串进行解析
    pub fn parse(meta: &str) -> Self {
        let root = json::parse(meta).unwrap();

        VersionMetaGroup(root.members().map(|e| VersionMeta::load(e)).collect())
    }

    /// 将所有版本元数据序列化成json字符串
    pub fn serialize(&self) -> String {
        let mut obj = JsonValue::new_array();

        for v in &self.0 {
            obj.push(v.serialize()).unwrap();
        }

        obj.pretty(4)
    }

    /// 往元数据组中添加一个新的版本元数据
    pub fn add(&mut self, meta: VersionMeta) {
        self.0.push(meta);
    }
}

impl<'a> IntoIterator for &'a VersionMetaGroup {
    type Item = &'a VersionMeta;

    type IntoIter = std::slice::Iter<'a, VersionMeta>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}