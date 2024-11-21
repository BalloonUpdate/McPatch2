//! 命令行主逻辑
//! 
//! 目前支持的功能：
//! 
//! + [x] 打包新版本
//! + [x] 合并新版本
//! + [x] 解压并测试现有的文件
//! + [x] 合并版本
//! + [x] 检查工作空间目录修改情况
//! + [ ] 启动内置服务端

pub mod webstate;
pub mod task_executor;
pub mod log;
pub mod cmd;
pub mod io;
pub mod fs;
pub mod file_status;
mod auth;
pub mod auth_layer;

use axum::routing::post;
use axum::Router;
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::web::auth_layer::AuthLayer;
use crate::web::cmd::check::api_check;
use crate::web::cmd::combine::api_combine;
use crate::web::cmd::pack::api_pack;
use crate::web::cmd::revert::api_revert;
use crate::web::cmd::test::api_test;
use crate::web::fs::delete::api_delete;
use crate::web::fs::disk_info::api_disk_info;
use crate::web::fs::download::api_download;
use crate::web::fs::list::api_list;
use crate::web::fs::make_directory::api_make_directory;
use crate::web::fs::upload::api_upload;
use crate::web::io::console_full::api_console_full;
use crate::web::io::console_more::api_console_more;
use crate::web::webstate::WebState;
use crate::AppContext;

#[derive(Serialize)]
pub struct PublicResponseBody<T> where T : Serialize {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

pub fn serve_web(ctx: AppContext) {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    
    runtime.block_on(async move {
        let webstate = WebState::new(ctx.to_owned());

        let app = Router::new()
            .route("/api/terminal/full", post(api_console_full))
            .route("/api/terminal/more", post(api_console_more))

            .route("/api/task/check", post(api_check))
            .route("/api/task/test", post(api_test))
            .route("/api/task/combine", post(api_combine))
            .route("/api/task/pack", post(api_pack))
            .route("/api/task/revert", post(api_revert))

            .route("/api/fs/disk-info", post(api_disk_info))
            .route("/api/fs/list", post(api_list))
            .route("/api/fs/upload", post(api_upload))
            .route("/api/fs/download", post(api_download))
            .route("/api/fs/make-directory", post(api_make_directory))
            .route("/api/fs/delete", post(api_delete))
            
            .layer(CorsLayer::permissive())
            .layer(AuthLayer::new(webstate.clone()))
            .with_state(webstate.clone())
            ;

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}
