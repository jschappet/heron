use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use chrono::Utc;

fn main() {
    let out_dir = "./src/".to_string();
    let path = PathBuf::from(out_dir).join("build_info.rs");

    let build_time = Utc::now().format("%Y-%m-%d-%H%M%S").to_string();
    let build_date = Utc::now().format("%Y-%m-%d").to_string();

    let mut file = File::create(path).unwrap();
    write!(
        file,
        "pub const BUILD_TIME: &str = \"{}\";\npub const BUILD_DATE: &str = \"{}\";\n",
        build_time, build_date
    )
    .unwrap();
}
