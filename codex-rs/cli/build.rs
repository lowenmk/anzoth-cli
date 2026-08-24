use std::{env, fs, path::PathBuf, process::Command};

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
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("out dir"));
    let rc_path = out_dir.join("anzoth-icon.rc");
    let res_path = out_dir.join("anzoth-icon.res");

    let rc_contents = format!(r#"1 ICON "{}""#, icon_path.display());
    fs::write(&rc_path, rc_contents).expect("write Windows resource script");

    let status = Command::new("rc")
        .args([
            "/nologo",
            "/fo",
            res_path
                .to_str()
                .expect("resource output path is valid UTF-8"),
            rc_path
                .to_str()
                .expect("resource script path is valid UTF-8"),
        ])
        .status()
        .expect("run rc.exe");
    assert!(
        status.success(),
        "rc.exe failed to compile Windows resources"
    );

    println!("cargo:rerun-if-changed={}", icon_path.display());
    println!("cargo:rustc-link-arg-bin=anzoth={}", res_path.display());
}
