#[macro_use]
extern crate log;

use std::error::Error;
use std::sync::{Arc, Mutex};

use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use bytes::Bytes;
use futures::SinkExt;

mod db;
mod parser;

fn setup_logger() {
    use env_logger::{Builder, Target};
    use log::LevelFilter;

    Builder::new()
        .filter(None, LevelFilter::Debug)
        .target(Target::Stdout)
        .format_timestamp_nanos()
        .init();
}

async fn handle_socket(db: Arc<Mutex<db::DB>>, socket: TcpStream) {
    debug!("accepting connection");

    let mut resps = vec!();

    let mut framer = Framed::new(socket, LengthDelimitedCodec::new());
    while let Some(result) = framer.next().await {
        match result {
            Ok(frame) => {
                let cmd = match parser::parse(&*frame) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        resps.push(format!("{:?}", e));
                        continue
                    },
                };

                match db.lock().unwrap().exec(cmd) {
                    Ok(result) => resps.push(result),
                    Err(e) => resps.push(format!("{:?}", e)),
                };
            }
            Err(e) => {
                error!("error decoding from socket: {}", e);
                break;
            }
        }
    }

    if let Err(e) = framer.send(Bytes::from(resps.join("\n"))).await {
        error!("could not respond: {}", e);
    }

    debug!("closing connection");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    info!("starting server");

    let db = Arc::new(Mutex::new(db::DB::new()));

    let addr = "0.0.0.0:4242".to_owned();
    let mut listener = TcpListener::bind(&addr).await?;
    info!("listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(handle_socket(db.clone(), socket));
            }
            Err(e) => error!("error accepting listener: {}", e),
        }

        // Temporary full-state debugging for very early protocol dev.
        // Will need to get extensive integration testing up asap.
        debug!("{:?}", db);
    }
}
