//! 用于生成一些方便手动测试的数据

use std::process::Command;
use store::{FsStore, Handle};

/// 生成一个 aplusb.zip
fn main() {
    server::dev::gen_test_fulldata()
        .save(&Handle::new("aplusb"))
        .unwrap();

    let r = Command::new("zip")
        .current_dir("aplusb")
        .arg("-r")
        .arg("../aplusb.zip")
        .arg(".")
        .spawn()
        .unwrap();
    let o = r.wait_with_output().unwrap();
    dbg!(o);

    let z = zip::ZipArchive::new(std::fs::File::open("aplusb.zip").unwrap()).unwrap();
    for name in z.file_names() {
        eprintln!("{}", name)
    }
    std::fs::remove_dir_all("aplusb").unwrap();
}
