//! 写入更新包

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::core::data::version_meta::FileChange;
use crate::core::data::version_meta_group::VersionMetaGroup;
use crate::utility::counted_write::CountedWrite;
use crate::utility::partial_read::PartialRead;

pub struct MetadataLocation {
    pub offset: u64,
    pub length: u64,
}

/// 代表一个更新包写入器，用于生成tar格式的更新包
pub struct TarWriter {
    builder: tar::Builder<CountedWrite<std::fs::File>>,
    addresses: HashMap<String, u64>,
    finished: bool,
}

impl TarWriter {
    /// 创建一个TarWriter，并将数据写到`file`文件中
    pub fn new(file: impl AsRef<Path>) -> Self {
        let open = std::fs::File::options().create(true).truncate(true).write(true).open(file).unwrap();

        Self {
            builder: tar::Builder::new(CountedWrite::new(open)), 
            addresses: HashMap::new(),
            finished: false,
        }
    }

    /// 往更新包里添加一个文件，除了数据和长度以外，还需要额外提供文件路径和所属版本号
    pub fn add_file(&mut self, mut data: impl Read, len: u64, path: &str, version: &str) {
        assert!(!self.finished, "TarWriter has already closed");

        // 写入更新包中
        let mut header = tar::Header::new_gnu();
        header.set_size(len);

        let partial_read = PartialRead::new(&mut data, len);
        self.builder.append_data(&mut header, path, partial_read).unwrap();

        let mut padding = 512 - (len % 512);

        if padding >= 512 {
            padding = 0;
        }

        let position = self.builder.get_ref().count();

        // println!(">>> {}: {}, {}, padding: {}", path, len, ptr, padding);
        
        // 记录当前数据偏移位置
        let key = format!("{}_{}", path, version);
        let tar_offset = position - len - padding;

        self.addresses.insert(key, tar_offset);
    }

    /// 完成更新包的创建，并返回元数据的偏移值和长度
    pub fn finish(mut self, mut meta_group: VersionMetaGroup) -> MetadataLocation {
        assert!(!self.finished, "TarWriter has already closed");

        // 更新元数据中的偏移值
        for meta in &mut meta_group {
            for change in meta.changes.iter_mut() {
                if let FileChange::UpdateFile { path, offset, .. } = change {
                    // 合并文件时，中间版本里的文件数据为了节省空间，是不存储的
                    // 也就是说即使这些元数据里有offset，len这些数据，但这些数据都是无效的
                    // 正常情况下客户端也不会去这个数据，如果读取了那么必定是数据受损了
                    let key = format!("{}_{}", path, &meta.label);
                    match self.addresses.get(&key) {
                        Some(addr) => *offset = *addr,
                        None => (),
                    }
                }
            }
        }

        // 序列化元数据组
        let metadata_offset = self.builder.get_ref().count();
        let file_content = meta_group.serialize();
        let file_content = file_content.as_bytes();

        // 写入元数据
        let mut header = tar::Header::new_gnu();
        header.set_size(file_content.len() as u64);
        self.builder.append_data(&mut header, "metadata.txt", std::io::Cursor::new(&file_content)).unwrap();

        // 写入完毕
        self.builder.finish().unwrap();
        self.finished = true;

        MetadataLocation {
            offset: metadata_offset + 512,
            length: file_content.len() as u64,
        }
    }
}

impl Drop for TarWriter {
    fn drop(&mut self) {
        assert!(self.finished, "TarWriter has not closed yet");
    }
}