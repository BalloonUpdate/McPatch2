//! 测试所有更新包

use crate::common::file_hash::calculate_hash;
use crate::common::tar_reader::TarReader;
use crate::data::index_file::IndexFile;
use crate::data::version_meta::FileChange;
use crate::AppContext;

/// 执行更新包解压测试
pub fn do_test(ctx: &AppContext) -> i32 {
    println!("正在执行更新包的解压测试");

    let index_file = IndexFile::load(&ctx.index_file_internal);

    // 读取现有更新包
    for v in &index_file {
        let mut reader = TarReader::new(ctx.public_dir.join(&v.filename));
        for meta in &reader.read_metadata_group(v.offset, v.len) {
            for change in &meta.changes {
                if let FileChange::UpdateFile { path, hash, offset, len, .. } = change {
                    println!("正在测试: {}({}) 的 {}", meta.label, v.filename, path);
    
                    let mut open = reader.open_file(*offset, *len);
                    let actual = calculate_hash(&mut open);
                    assert!(&actual == hash, "hashes do not match, path: {} actual: {}, expected: {}", path, actual, hash);
                }
            }
        }
        
    }

    println!("测试通过！");

    0
}