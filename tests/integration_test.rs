/// All tests in this folder assume a server running on localhost:4242
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use bytes::Bytes;

static LOCAL_REMITS: &str = "localhost:4242";

macro_rules! should_respond_with {
    ($framer:expr, $bytes:expr, $resp:expr) => {
        $framer
            .send(Bytes::from($bytes))
            .await
            .expect("could not send command");

        let result = $framer
            .next()
            .await
            .expect("no response from remits")
            .expect("could not understand response");

        assert_eq!(&*result, $resp);
    };
}

async fn connect_to_remits() -> Framed<TcpStream, LengthDelimitedCodec> {
    let stream = TcpStream::connect(LOCAL_REMITS)
        .await
        .expect("could not connect to localhost:4242");

    Framed::new(stream, LengthDelimitedCodec::new())
}

#[tokio::test]
async fn test_can_connect_to_server() {
    connect_to_remits().await;
}

#[tokio::test]
async fn test_can_create_log() {
    let mut framer = connect_to_remits().await;
    should_respond_with!(framer, "LOG ADD test_log", b"+ok");

    // second create should be a noop, but still respond with "ok"
    should_respond_with!(framer, "LOG ADD test_log", b"+ok");
}

#[tokio::test]
async fn test_can_create_itr() {
    // Should succeed in the happy path
    let mut framer = connect_to_remits().await;
    let itr_cmd = "ITR ADD test_log test_itr map \n return msg";
    should_respond_with!(framer, itr_cmd, b"+ok");

    // Should fail if invalid itr typee
    let mut framer = connect_to_remits().await;
    let itr_cmd = "ITR ADD test_log test_itr NOT_A_ITR_TYPE \n return msg";
    should_respond_with!(framer, itr_cmd, b"!ItrTypeInvalid");
}

#[tokio::test]
async fn test_malformed_msg_add() {
    let mut framer = connect_to_remits().await;
    should_respond_with!(framer, "LOG ADD test_log", b"+ok");

    // Not valid message pack
    let msg_cmd = b"MSG ADD test_log \x93\x00\x2a".to_vec();
    should_respond_with!(framer, msg_cmd, b"!MsgNotValidMessagePack");
}
