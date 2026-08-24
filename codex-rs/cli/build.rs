extern crate embed_resource;

use std::{env, fs, path::PathBuf};

fn main() {
    match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("macos") => println!("cargo:rustc-link-arg=-ObjC"),
        Ok("windows") => embed_windows_icon(),
        _ => {}
    }
}

fn embed_windows_icon() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let icon_path = manifest_dir.join("../../assets/windows/anzoth.ico");
    let icon_path = icon_path
        .canonicalize()
        .expect("resolve canonical Windows icon path");
    let icon_path = icon_path.to_string_lossy().replace('\\', "/");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("out dir"));
    let rc_path = out_dir.join("anzoth-icon.rc");

    let rc_contents = format!(r#"1 ICON "{}""#, icon_path);
    fs::write(&rc_path, rc_contents).expect("write Windows resource script");

    println!("cargo:rerun-if-changed={}", icon_path);
    embed_resource::compile_for(rc_path, &["anzoth"], embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}
