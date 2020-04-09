#[macro_use]
extern crate log;
#[macro_use]
extern crate num_derive;

use protocol::Connection;
use std::sync::Arc;
use tokio::net::TcpListener;

mod commands;
mod config;
mod db;
mod errors;
mod protocol;

async fn handle(db: Arc<db::DB>, mut conn: Connection) {
    debug!("accepting connection");

    while let Some(res) = conn.next_request().await {
        let cmd = match res {
            Ok(cmd) => cmd,
            Err(e) => {
                conn.respond(e.into()).await;
                continue;
            }
        };
        debug!("received command: {:?}", &cmd);

        let resp = db.exec(cmd);
        conn.respond(resp).await;
    }

    debug!("closing connection");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load();

    info!("starting server");
    let mut listener = TcpListener::bind(cfg.addr()).await?;
    info!("listening on {}", cfg.addr());

    let db = Arc::new(db::DB::new());
    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(handle(db.clone(), socket.into()));
            }
            Err(e) => error!("error accepting listener: {}", e),
        }
    }
}
