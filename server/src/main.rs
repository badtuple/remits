#[macro_use]
extern crate log;

mod config;
mod protocol;

use config::Config;
use protocol::Body;

use env_logger::{Builder, Target};
use serde_json::Deserializer;
use storage::Storage;

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn setup_logger(log_level: &str) {
    Builder::new()
        .parse_filters(log_level)
        .target(Target::Stdout)
        .format_timestamp_nanos()
        .init();

    info!("log level set to {}", log_level);
}

fn main() {
    let cfg = match Config::load("TODO: Path does not work yet") {
        Ok(cfg) => cfg,
        Err(e) => panic!("could not load config: {:?}", e),
    };

    setup_logger(&*cfg.log_level);

    info!("opening storage");
    let storage = match Storage::open("TODO: Path does not work yet") {
        Ok(s) => s,
        Err(e) => return error!("could not open storage: {:?}", e),
    };

    // TODO: We're using a global mutex now for convenience during initial development.
    // There's no way we actually want a global mutex, but finding the correct latch granularity is
    // pointless until we have something working and a reliable benchmark suite.
    let storage = Arc::new(Mutex::new(storage));

    info!("starting server bound to {}", cfg.addr());
    let listener = match TcpListener::bind(cfg.addr()) {
        Ok(l) => l,
        Err(e) => return error!("could not start tcp server: {}", e),
    };

    info!("listening");

    let mut connection_counter: u64 = 0;
    for stream in listener.incoming() {
        connection_counter += 1;
        let conn_id = *(&connection_counter); // hack to force copy of id
        let storage = storage.clone();

        match stream {
            Ok(s) => {
                // TODO: we're gonna want to move to async eventually I'm pretty sure, but using a
                // thread per connection model for simplicity right now. Will move it over once
                // things are working and we have a reliable benchmark suite. No need opting into
                // that complexity if we can't prove it helps us.
                thread::spawn(move || handle_stream(conn_id, storage, s));
            }
            Err(e) => error!("could not accept stream: {}", e),
        }
    }
}

fn handle_stream(conn_id: u64, _storage: Arc<Mutex<Storage>>, conn: TcpStream) {
    info!("accepted stream {}", conn_id);

    let stream = Deserializer::from_reader(conn).into_iter::<Body>();
    for result in stream {
        match result {
            Ok(body) => info!("recvd body on conn_id {}: {:?}", conn_id, body),
            Err(e) => {
                error!("recvd malformed body on conn_id {}: {:?}", conn_id, e);
                return;
            }
        }
    }

    info!("closing stream {}", conn_id);
}
