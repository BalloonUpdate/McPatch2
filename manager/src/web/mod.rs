pub mod api;
pub mod file_status;
pub mod log;
pub mod webstate;
pub mod task_executor;
pub mod auth_layer;
pub mod token;

use axum::routing::post;
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::config::config::Config;
use crate::web::api::task::check::api_check;
use crate::web::api::task::combine::api_combine;
use crate::web::api::task::pack::api_pack;
use crate::web::api::task::revert::api_revert;
use crate::web::api::task::test::api_test;
use crate::web::api::fs::delete::api_delete;
use crate::web::api::fs::disk_info::api_disk_info;
use crate::web::api::fs::download::api_download;
use crate::web::api::fs::list::api_list;
use crate::web::api::fs::make_directory::api_make_directory;
use crate::web::api::fs::upload::api_upload;
use crate::web::api::terminal::full::api_full;
use crate::web::api::terminal::more::api_more;
use crate::web::api::user::change_password::api_change_password;
use crate::web::api::user::change_username::api_change_username;
use crate::web::api::user::login::api_login;
use crate::web::auth_layer::AuthLayer;
use crate::web::webstate::WebState;

pub fn serve_web() {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    
    runtime.block_on(async move {
        let (config, first_run) = Config::load();

        if first_run {
            let mut lock = config.config.lock().await;
            println!("检测到首次运行，正在生成配置信息。");
            println!("这是账号和密码，请务必牢记。账号：admin，密码：{}（只会显示一次，请注意保存）", lock.web.password);

            // 将密码进行hash计算
            let raw = lock.web.password.to_owned();
            lock.web.set_password(&raw);
            drop(lock);

            config.save_async().await;
        }

        let lock = config.config.lock().await;
        let listen_addr = lock.web.serve_listen_addr.to_owned();
        let listen_port = lock.web.serve_listen_port.to_owned();
        let listen = format!("{}:{}", listen_addr, listen_port);
        drop(lock);
        
        println!("web监听地址和端口：{}", listen);

        let webstate = WebState::new(config);

        let app = Router::new()
            // 这部分参与请求验证
            .route("/api/user/logout", post(api_login))
            .route("/api/user/change-username", post(api_change_username))
            .route("/api/user/change-password", post(api_change_password))

            .route("/api/terminal/full", post(api_full))
            .route("/api/terminal/more", post(api_more))

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
            .route_layer(AuthLayer::new(webstate.clone()))

            // 这部分不参与请求验证
            .route("/api/user/login", post(api_login))
            
            // 其它的中间件
            .layer(CorsLayer::permissive())
            .with_state(webstate.clone())
            ;

        let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}
