use std::path::PathBuf;

fn main() {
    // 为windows平台增加pe文件版本号信息
    let rc_file = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("pe.rc");

    println!("cargo:rerun-if-changed={}", rc_file.to_str().unwrap());

    let major = env!("CARGO_PKG_VERSION_MAJOR");
    let minor = env!("CARGO_PKG_VERSION_MINOR");
    let patch = env!("CARGO_PKG_VERSION_PATCH");
    let pre = if env!("CARGO_PKG_VERSION_PRE").is_empty() { "0" } else { env!("CARGO_PKG_VERSION_PRE") };

    println!("1 VERSIONINFO FILEVERSION {major},{minor},{patch},{pre} {{ }}");

    let rc_content = format!("1 VERSIONINFO FILEVERSION {major},{minor},{patch},{pre} {{ }}");

    std::fs::write(&rc_file, rc_content).unwrap();

    embed_resource::compile(&rc_file, embed_resource::NONE);
}