pub mod traffic_control;
pub mod counted_write;
pub mod to_detail_error;
pub mod io_utils;
pub mod partial_read;
pub mod filename_ext;
pub mod vec_ext;

/// 判断是否在cargo环境中运行
pub fn is_running_under_cargo() -> bool {
    #[cfg(debug_assertions)]
    let result = std::env::vars().any(|p| p.0.eq_ignore_ascii_case("CARGO"));

    #[cfg(not(debug_assertions))]
    let result = false;

    result
}

/// 将一个`iter`所有内容连接成字符串，分隔符是`split`
pub fn join_string(iter: impl Iterator<Item = impl AsRef<str>>, split: &str) -> String {
    let mut result = String::new();
    let mut insert = false;

    for e in iter {
        if insert {
            result.push_str(split);
        }

        result.push_str(e.as_ref());
        insert = true;
    }

    result
}

