use axum::extract::State;
use axum::response::Response;
use shared::data::index_file::IndexFile;

use crate::common::archive_tester::ArchiveTester;
use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

/// 执行更新包解压测试
pub async fn api_test(State(state): State<WebState>) -> Response {
    state.clone().te.lock().await
        .try_schedule(move || do_test(state)).await;

    PublicResponseBody::<()>::ok_no_data()
}

fn do_test(state: WebState) {
    let config = state.config;
    let mut console = state.console.blocking_lock();

    console.log_debug("正在执行更新包的解压测试");

    let index_file = IndexFile::load_from_file(&config.index_file);

    let mut tester = ArchiveTester::new();

    // 读取现有更新包
    for v in &index_file {
        tester.feed(config.public_dir.join(&v.filename), v.offset, v.len);
    }

    // 执行测试
    tester.finish(|e| console.log_debug(format!("{}/{} 正在测试 {} 的 {} ({}+{})", e.index, e.total, e.label, e.path, e.offset, e.len))).unwrap();

    console.log_info("测试通过！");
}