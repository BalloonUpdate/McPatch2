use std::io::Read;
use std::io::Seek;
use std::path::Path;

use crate::data::version_meta::VersionMeta;

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

pub struct VersionReader {
    open: std::fs::File
}

impl VersionReader {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let open = std::fs::File::open(path).unwrap();
    
        Self { open }
    }

    pub fn read_metadata(&mut self) -> VersionMeta {
        let mut open = &self.open;

        // 读取地址文件
        let mut addr_buf = [0u8; 512];
        open.seek(std::io::SeekFrom::Start(512)).unwrap();
        open.read_exact(&mut addr_buf).unwrap();
        let content = std::str::from_utf8(&addr_buf).unwrap();
        let meta_offset = u64::from_str_radix(content.trim(), 10).unwrap();
        
        // 读取元数据的Header
        let mut header = tar::Header::new_gnu();
        open.seek(std::io::SeekFrom::Start(meta_offset)).unwrap();
        open.read_exact(header.as_mut_bytes()).unwrap();
        let meta_len = header.size().unwrap();
        assert!(meta_len <= usize::MAX as u64);

        // 读取元数据
        let mut buf = Vec::<u8>::new();
        buf.resize(meta_len as usize, 0);
        open.read_exact(&mut buf).unwrap();
        let meta_content = std::str::from_utf8(&buf).unwrap();

        // 解析元数据
        VersionMeta::load(meta_content)
    }

    pub fn open_file(&mut self, offset: u64, len: u64) -> LimitedRead<std::fs::File> {
        self.open.seek(std::io::SeekFrom::Start(offset)).unwrap();

        LimitedRead(&mut self.open, len)
    }
}
