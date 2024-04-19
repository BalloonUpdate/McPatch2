//! 读取更新包

use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;

use mcpatch_shared::data::version_meta_group::VersionMetaGroup;
use mcpatch_shared::utility::partial_read::PartialRead;

/// 代表一个更新包读取器，用于读取tar格式的更新包里面的数据
pub struct TarReader {
    open: std::fs::File
}

impl TarReader {
    /// 创建一个TarReader，从`file`读取数据
    pub fn new(file: impl AsRef<Path>) -> Self {
        Self { open: std::fs::File::open(file).unwrap() }
    }

    /// 读取更新包中的元数据，需要提供元数据的`offset`和`len`以便定位
    pub fn read_metadata_group(&mut self, offset: u64, len: u32) -> VersionMetaGroup {
        let mut buf = Vec::<u8>::new();
        buf.resize(len as usize, 0);

        self.open.seek(SeekFrom::Start(offset)).unwrap();
        self.open.read_exact(&mut buf).unwrap();

        VersionMetaGroup::parse(std::str::from_utf8(&buf).unwrap())
    }

    /// 读取更新包中的一个文件数据，需要提供文件的`offset`和`len`以便定位
    pub fn open_file(&mut self, offset: u64, len: u64) -> PartialRead<std::fs::File> {
        self.open.seek(SeekFrom::Start(offset)).unwrap();

        PartialRead::new(&mut self.open, len)
    }
}
