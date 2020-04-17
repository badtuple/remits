use db::DB;
use protocol::Connection;
use std::sync::Arc;
use tokio::net::TcpListener;
use config::RemitsConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load();
    info!("{:?}", &cfg);
    run_server(cfg);
    Ok(())
}
