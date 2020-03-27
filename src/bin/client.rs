use std::io;
use std::io::Write;

use futures::SinkExt;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

static LOCAL_REMITS: &str = "localhost:4242";

fn print_prompt() {
    print!("remits> ");
    io::stdout().flush().expect("could not write to stdout");
}

async fn connect_to_remits() -> TcpStream {
    TcpStream::connect(LOCAL_REMITS)
        .await
        .expect("could not open tcp stream to localhost:4242")
}

#[tokio::main]
async fn main() {
    // setup
    let stream = connect_to_remits().await;
    let mut framer = Framed::new(stream, LengthDelimitedCodec::new());

    // start REPL
    print_prompt();

    let stdin = io::stdin();
    let mut buffer = String::new();
    while stdin
        .read_line(&mut buffer)
        .expect("unable to read from stdin")
        > 0
    {
        let input = buffer.trim().to_owned();
        match framer.send(input.into()).await {
            Ok(_) => {
                match framer.next().await {
                    Some(result) => {
                        let res = format!("{:?}", result.unwrap_or_else(|_| "".into()));
                        if res.chars().next().unwrap() == '!' {
                            println!(
                                "Error {}",
                                res.char_indices()
                                    .nth(1)
                                    .and_then(|(i, _)| res.get(i..))
                                    .unwrap_or("")
                            )
                        } else {
                            println!(
                                "{}",
                                res.char_indices()
                                    .nth(1)
                                    .and_then(|(i, _)| res.get(i..))
                                    .unwrap_or("")
                            )
                        }
                    }
                    None => eprintln!("no response from remits"),
                };
            }
            Err(e) => eprintln!("could not send to remits: {}", e),
        };

        buffer.clear();
        print_prompt();
    }
}
