use std::{thread, time};
use tempfile::TempDir;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub async fn start_server() {
    // Create a directory inside of `std::env::temp_dir()`.
    let tmp_dir = TempDir::new().unwrap();
    let file_path = tmp_dir.path().to_str().to_owned().unwrap();
    //dir.path().to_str().unwrap().to_owned();

    let cfg = remitslib::config::RemitsConfig {
        port: Some("4243".into()),
        log_level: Some("trace".into()),
        db_path: Some(file_path.into()),
    };

    remitslib::server::run_server(cfg).await;
}
