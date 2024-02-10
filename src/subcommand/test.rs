use crate::common::file_hash::calculate_hash;
use crate::common::version_reader::VersionReader;
use crate::data::index_file::IndexFile;
use crate::data::version_meta::FileChange;
use crate::AppContext;

pub fn do_test(ctx: AppContext) -> i32 {
    println!("正在执行更新包的解压测试");

    let index_file = IndexFile::load(&ctx.index_file_internal);

    // 读取现有更新包
    for filename in &index_file.versions {
        let mut reader = VersionReader::new(ctx.public_dir.join(filename));
        let meta = reader.read_metadata();
        
        for change in &meta.changes {
            if let FileChange::UpdateFile { path, hash, offset, len, .. } = change {
                println!("正在测试: {} 的 {}", filename, path);

                let mut open = reader.open_file(*offset, *len);
                let actual = calculate_hash(&mut open);
                assert!(&actual != hash, "hashes do not match, path: {} actual: {}, expected: {}", path, actual, hash);
            }
        }
    }

    println!("测试通过！");

    0
}