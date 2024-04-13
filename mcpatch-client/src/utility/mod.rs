pub mod limited_read_async;

/// 判断是否在cargo环境中运行
pub fn is_running_under_cargo() -> bool {
    std::env::vars().any(|p| p.0.eq_ignore_ascii_case("CARGO"))
}