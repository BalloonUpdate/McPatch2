//! 测试所有更新包

use crate::common::archive_tester::ArchiveTester;
use crate::data::index_file::IndexFile;
use crate::AppContext;

/// 执行更新包解压测试
pub fn do_test(ctx: &AppContext) -> i32 {
    println!("正在执行更新包的解压测试");

    let index_file = IndexFile::load(&ctx.index_file);

    let mut tester = ArchiveTester::new();

    // 读取现有更新包
    for v in &index_file {
        tester.feed(ctx.public_dir.join(&v.filename), v.offset, v.len);
    }

    // 执行测试
    tester.finish();

    println!("测试通过！");

    0
}
