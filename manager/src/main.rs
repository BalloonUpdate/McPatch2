//! mcpatch2管理端第二版

use crate::web::serve_web;
pub mod utility;
pub mod subcommand;
pub mod diff;
pub mod common;
pub mod config;
pub mod web;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    serve_web();
}