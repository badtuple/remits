#[macro_use]
extern crate log;
#[macro_use]
extern crate num_derive;

use argh::FromArgs;
use env_logger::{Builder, Target};
use protocol::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

mod commands;
mod db;
mod errors;
mod protocol;

/// Server options
#[derive(Clone, Debug, Serialize, Deserialize, FromArgs)]
struct RemitsConfig {
    #[argh(option, short = 'p')]
    /// what port to start remits on
    pub port: Option<String>,
    // v can change dont care
    #[argh(option, short = 'v')]
    /// verbosity of logs
    pub log_level: Option<String>,
}

impl RemitsConfig {
    fn update_from_flags(&mut self) {
        let flags: RemitsConfig = argh::from_env();
        // This one must be first so debug logs work the rest of the way down
        setup_logger(self.log_level.clone(), flags.log_level);
        if flags.port.is_some() {
            debug!(
                "Replacing config option \"port\":{} with flag \"-p/--port\":{}",
                self.port.as_ref().unwrap(),
                flags.port.as_ref().unwrap()
            );
            self.port = flags.port;
        }
    }
}

impl ::std::default::Default for RemitsConfig {
    fn default() -> Self {
        Self {
            port: Some("4242".into()),
            log_level: Some("info".into()),
        }
    }
}
fn setup_logger(config_level: Option<String>, flag_level: Option<String>) {
    let log_level = &flag_level.unwrap_or(
        config_level
            .as_ref()
            .unwrap_or(&"info".to_owned())
            .to_string(),
    );
    Builder::new()
        .parse_filters(log_level)
        .target(Target::Stdout)
        .format_timestamp_nanos()
        .init();
    debug!("Log level set to {}", log_level);
}

async fn handle(db: Arc<Mutex<db::DB>>, mut conn: Connection) {
    debug!("accepting connection");

    while let Some(res) = conn.next_request().await {
        let cmd = match res {
            Ok(cmd) => cmd,
            Err(e) => {
                conn.respond(e.into()).await;
                continue;
            }
        };

        let resp = db.lock().unwrap().exec(cmd);
        conn.respond(resp).await;
    }

    debug!("closing connection");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg: RemitsConfig = confy::load("remits")?;
    cfg.update_from_flags();

    info!("starting server");

    let db = Arc::new(Mutex::new(db::DB::new()));

    let addr = "0.0.0.0:".to_owned() + &cfg.port.expect("No port defined");
    let mut listener = TcpListener::bind(&addr).await?;
    info!("listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(handle(db.clone(), socket.into()));
            }
            Err(e) => error!("error accepting listener: {}", e),
        }
    }
}
