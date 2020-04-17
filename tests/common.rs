use std::process::{Child, Command};
use tempfile::TempDir;
extern crate remits;
use remits;

pub async fn start_server() {
    // Create a directory inside of `std::env::temp_dir()`.
    let tmp_dir = TempDir::new().unwrap();
    let file_path = tmp_dir.path().to_str().to_owned().unwrap();
    //dir.path().to_str().unwrap().to_owned();

    let cfg = config::RemitsConfig{

    };

    run_server(cfg);
}
pub fn teardown(server: &Child) {
    Command::new("kill")
        .arg("-9")
        .arg(server.id().to_string())
        .output()
        .unwrap();
}
