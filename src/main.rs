#[macro_use]
extern crate log;

use argh::FromArgs;
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use futures::SinkExt;

mod db;
mod parser;

// Need a better place to store these so they are searchable and not opaque to contributors 
macro_rules! format_error_response {
    ($err:expr) => {{
        let mut out: BytesMut = BytesMut::from("!");
        out.extend_from_slice(&Bytes::from($err));
        out.into()
    }};
}

macro_rules! format_success_response {
    ($resp:expr) => {{
        let mut out: BytesMut = BytesMut::from("+");
        out.extend_from_slice(&Bytes::from($resp));
        out.into()
    }};
}

#[derive(Debug, Serialize, Deserialize, FromArgs)]
/// Top level config
struct RemitsConfig {
    #[argh(option, short = 'p', default = "\"4242\".to_owned()")]
    /// what port to start remits on
    pub port: String,
}

/// `RemitsConfig` implements `Default`
impl ::std::default::Default for RemitsConfig {
    fn default() -> Self {
        Self {
            port: "4242".into(),
        }
    }
}
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

    let mut framer = Framed::new(socket, LengthDelimitedCodec::new());

    while let Some(result) = framer.next().await {
        let frame = match result {
            Ok(f) => f,
            Err(e) => {
                error!("could not read from socket: {}", e);
                break;
            }
        };

        debug!("received command: {:?}", &frame);
        let cmd = match parser::parse(&*frame) {
            Ok(cmd) => cmd,
            Err(e) => {
                debug!("responding with: {:?}", e);
                let resp: Bytes = format_error_response!(e);

                let _ = framer.send(resp).await;
                continue;
            }
        };

        let out = db.lock().unwrap().exec(cmd);
        let resp = match out {
            Ok(res) => format_success_response!(res),
            Err(e) => format_error_response!(e),
        };

        debug!("responding with: {:?}", resp);
        if let Err(e) = framer.send(resp).await {
            error!("could not respond: {}", e);
        }
    }

    debug!("closing connection");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let mut cfg: RemitsConfig = confy::load("remits")?;
    cfg = argh::from_env();
    info!("starting server");

    let db = Arc::new(Mutex::new(db::DB::new()));

    let addr = "0.0.0.0:".to_owned() + &cfg.port;
    let mut listener = TcpListener::bind(&addr).await?;
    info!("listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(handle_socket(db.clone(), socket));
            }
            Err(e) => error!("error accepting listener: {}", e),
        }
    }
}
