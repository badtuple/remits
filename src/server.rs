use crate::config::RemitsConfig;
use crate::db::DB;
use crate::protocol::Connection;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn handle(db: Arc<DB>, mut conn: Connection) {
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

pub async fn run_server(cfg: RemitsConfig) {
    info!("starting server");
    let mut listener = TcpListener::bind(cfg.addr()).await.unwrap();
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
