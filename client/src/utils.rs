pub fn convert_bytes(b: u64) -> String {
    if b < 1024 {
        return format!("{} B", b);
    }

    if b < 1024 * 1024 {
        return format!("{:.1} KB", b as f32 / 1024.0);
    }

    if b < 1024 * 1024 * 1024 {
        return format!("{:.1} MB", b as f32 / 1024.0 / 1024.0);
    }

    return format!("{:.1} GB", b as f32 / 1024.0 / 1024.0 / 1024.0);
}