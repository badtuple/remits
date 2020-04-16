use std::process::{Child, Command};
use std::{thread, time};
use tempfile::TempDir;

pub fn start_server() -> Child {
    // Create a directory inside of `std::env::temp_dir()`.
    let tmp_dir = TempDir::new().unwrap();
    let file_path = tmp_dir.path().to_str().to_owned().unwrap();
    //dir.path().to_str().unwrap().to_owned();

    let child = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("-p")
        .arg("4243")
        .arg("-d")
        .arg(file_path)
        .spawn()
        .unwrap();

    let five_secs = time::Duration::from_secs(5);

    thread::sleep(five_secs);

    child
}
pub fn teardown(server: &Child) {
    Command::new("kill")
        .arg("-9")
        .arg(server.id().to_string())
        .output()
        .unwrap();
}
