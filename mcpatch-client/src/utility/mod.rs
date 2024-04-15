pub fn join_string(iter: impl Iterator<Item = impl AsRef<str>>, split: &str) -> String {
    let mut result = String::new();

    for e in iter {
        result.push_str(e.as_ref());
        result.push_str(split);
    }

    if result.ends_with(split) {
        result = result[0..result.rfind(split).unwrap()].to_owned();
    }

    result
}

/// 判断是否在cargo环境中运行
pub fn is_running_under_cargo() -> bool {
    std::env::vars().any(|p| p.0.eq_ignore_ascii_case("CARGO"))
}