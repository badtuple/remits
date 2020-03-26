/// All tests in this folder assume a server running on localhost:4242
use std::io;
use std::io::Write;

use futures::SinkExt;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use bytes::Bytes;

static LOCAL_REMITS: &str = "localhost:4242";

#[tokio::test]
async fn test_can_connect_to_server() {
    TcpStream::connect(LOCAL_REMITS)
        .await
        .expect("could not connect to localhost:4242");
}

#[tokio::test]
async fn test_can_create_log() {
    let stream = TcpStream::connect(LOCAL_REMITS)
        .await
        .expect("could not connect to localhost:4242");

    let mut framer = Framed::new(stream, LengthDelimitedCodec::new());

    framer
        .send(Bytes::from("LOG ADD test_log"))
        .await
        .expect("could not sent command");

    let result = framer
        .next()
        .await
        .expect("no response from remits")
        .expect("could not understand response");

    assert_eq!(&*result, b"ok");
}
