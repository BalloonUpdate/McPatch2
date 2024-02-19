use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;

use crate::data::version_meta_group::VersionMetaGroup;
use crate::utility::limited_read::LimitedRead;

/// 代表一个tar包读取器，用于读取tar格式的更新包里面的数据
pub struct TarReader {
    open: std::fs::File
}

impl TarReader {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let open = std::fs::File::open(path).unwrap();
    
        Self { open }
    }

    pub fn read_metadata_group(&mut self, meta_offset: u64, meta_len: u32) -> VersionMetaGroup {
        let mut open = &self.open;
        
        // 读取元数据
        let mut buf = Vec::<u8>::new();
        buf.resize(meta_len as usize, 0);

        open.seek(SeekFrom::Start(meta_offset)).unwrap();
        open.read_exact(&mut buf).unwrap();
        let meta_content = std::str::from_utf8(&buf).unwrap();

        // 解析元数据
        VersionMetaGroup::parse(meta_content)
    }

    /// 打开一个tar包中文件的Read对象
    pub fn open_file(&mut self, offset: u64, len: u64) -> LimitedRead<std::fs::File> {
        self.open.seek(SeekFrom::Start(offset)).unwrap();

        LimitedRead::new(&mut self.open, len)
    }
}
