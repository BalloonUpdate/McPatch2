use std::io::Read;
use std::io::Seek;
use std::path::Path;

use crate::data::version_meta_group::VersionMetaGroup;

pub struct LimitedRead<'a, R: Read>(&'a mut R, u64);

// impl<'a, R: Read> LimitedRead<'a, R> {
//     pub fn new(read: &'a mut R, limit: u64) -> Self {
//         Self(read, limit)
//     }
// }

impl<R: Read> Read for LimitedRead<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.1 == 0 {
            return Ok(0);
        }
        let consume = self.1.min(buf.len() as u64);
        self.1 -= consume;
        self.0.read(&mut buf[0..consume as usize])
    }
}

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

        open.seek(std::io::SeekFrom::Start(meta_offset)).unwrap();
        open.read_exact(&mut buf).unwrap();
        let meta_content = std::str::from_utf8(&buf).unwrap();

        // 解析元数据
        VersionMetaGroup::parse(meta_content)
    }

    pub fn open_file(&mut self, offset: u64, len: u64) -> LimitedRead<std::fs::File> {
        self.open.seek(std::io::SeekFrom::Start(offset)).unwrap();

        LimitedRead(&mut self.open, len)
    }
}
