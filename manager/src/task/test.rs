use crate::app_path::AppPath;
use crate::config::Config;
use crate::core::archive_tester::ArchiveTester;
use crate::core::data::index_file::IndexFile;
use crate::web::log::Console;


pub fn task_test(apppath: &AppPath, _config: &Config, console: &Console) -> u8 {
    console.log_debug("正在执行更新包的解压测试");

    let index_file = IndexFile::load_from_file(&apppath.index_file);

    let mut tester = ArchiveTester::new();

    // 读取现有更新包
    for v in &index_file {
        tester.feed(apppath.public_dir.join(&v.filename), v.offset, v.len);
    }

    // 执行测试
    tester.finish(|e| console.log_debug(format!("{}/{} 正在测试 {} 的 {} ({}+{})", e.index, e.total, e.label, e.path, e.offset, e.len))).unwrap();

    console.log_info("测试通过！");

    0
}