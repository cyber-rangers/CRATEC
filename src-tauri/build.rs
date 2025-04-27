use std::env;
use std::fs::File;
use std::io::Write;
use chrono::Local;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let now = Local::now().to_rfc3339();
    let mut f = File::create(format!("{}/build_info.rs", out_dir)).unwrap();
    write!(f, "pub const BUILD_DATE: &str = \"{}\";", now).unwrap();

    tauri_build::build();
}