//! mcpatch2管理端第二版

use crate::web::serve_web;
pub mod utility;
pub mod diff;
pub mod common;
pub mod config;
pub mod web;
pub mod builtin_server;
pub mod upload;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    serve_web();
}