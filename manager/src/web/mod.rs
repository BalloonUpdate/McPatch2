pub mod api;
pub mod file_status;
pub mod log;
pub mod webstate;
pub mod task_executor;
pub mod auth_layer;
pub mod token;

use std::net::SocketAddr;
use std::str::FromStr;

use axum::routing::get;
use axum::routing::post;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use tower_http::cors::CorsLayer;

use crate::config::config::Config;
use crate::web::api::fs::extract_file::api_extract_file;
use crate::web::api::fs::sign_file::api_sign_file;
use crate::web::api::public::api_public;
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
use crate::web::api::user::check_token::api_check_token;
use crate::web::api::user::login::api_login;
use crate::web::api::user::logout::api_logout;
use crate::web::auth_layer::AuthLayer;
use crate::web::webstate::WebState;

pub fn serve_web() {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    
    runtime.block_on(async move {
        let (config, first_run) = Config::load();

        if first_run {
            let mut lock = config.config.lock().await;
            println!("检测到首次运行，正在生成配置信息。");
            println!("这是账号和密码，请务必牢记。账号：admin，密码：{}（只会显示一次，请注意保存）", lock.user.password);

            // 将密码进行hash计算
            let raw = lock.user.password.to_owned();
            lock.user.set_password(&raw);
            drop(lock);

            config.save_async().await;
        }

        let lock = config.config.lock().await;
        let listen_addr = lock.web.serve_listen_addr.to_owned();
        let listen_port = lock.web.serve_listen_port.to_owned();
        let listen = format!("{}:{}", listen_addr, listen_port);
        
        println!("web监听地址和端口：{}", listen);

        // 配置tls
        let tls_config = if !lock.web.tls_cert_file.is_empty() && !lock.web.tls_key_file.is_empty() {
            // println!("准备加载TLS证书");

            let cert_file = config.working_dir.join(&lock.web.tls_cert_file);
            let key_file = config.working_dir.join(&lock.web.tls_key_file);
    
            if !cert_file.exists() || !key_file.exists() {
                if !cert_file.exists() {
                    println!("TLS cert文件找不到：{}", lock.web.tls_cert_file);
                }

                if !key_file.exists() {
                    println!("TLS key文件找不到：{}", lock.web.tls_key_file);
                }

                None
            } else {
                println!("TLS加密已启用");

                let tls_config = RustlsConfig::from_pem_file(cert_file, key_file)
                    .await
                    .unwrap();

                Some(tls_config)
            }
        } else {
            None
        };

        drop(lock);

        let webstate = WebState::new(config);

        let app = Router::new()
            // 这部分参与请求验证
            .route("/api/user/check-token", post(api_check_token))
            .route("/api/user/logout", post(api_logout))
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
            .route("/api/fs/sign-file", post(api_sign_file))
            .route_layer(AuthLayer::new(webstate.clone()))

            // 这部分不参与请求验证
            .route("/api/user/login", post(api_login))

            .route("/api/fs/extract-file", get(api_extract_file))

            .route("/public/*path", get(api_public))
            
            // 其它的中间件
            .layer(CorsLayer::permissive())
            .with_state(webstate.clone())
            ;

        // let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
        // axum::serve(listener, app).await.unwrap();

        let addr = SocketAddr::from_str(&listen).unwrap();

        match tls_config {
            Some(ok) => {
                axum_server::bind_rustls(addr, ok)
                    .serve(app.into_make_service())
                    .await
                    .unwrap();
            },
            None => {
                axum_server::bind(addr)
                    .serve(app.into_make_service())
                    .await
                    .unwrap();
            },
        }
    });
}
