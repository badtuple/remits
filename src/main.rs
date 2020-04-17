#[macro_use]
extern crate log;

use remitslib::config;
use remitslib::server::run_server;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load();
    info!("{:?}", &cfg);
    run_server(cfg);
    Ok(())
}