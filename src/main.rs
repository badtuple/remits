#[macro_use]
extern crate log;
#[macro_use]
extern crate num_derive;

use db::DB;
use protocol::Connection;
use std::sync::Arc;
use tokio::net::TcpListener;

mod commands;
mod config;
mod db;
mod errors;
mod protocol;

#[cfg(test)]
mod test_util;

async fn handle(db: Arc<DB>, mut conn: Connection) {
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
    info!("{:?}", &cfg);

    info!("starting server");
    let mut listener = TcpListener::bind(cfg.addr()).await?;
    info!("listening on {}", cfg.addr());

    let db = Arc::new(DB::new(cfg.db_path.unwrap()));

    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(handle(db.clone(), socket.into()));
            }
            Err(e) => error!("error accepting listener: {}", e),
        }
    }
}
