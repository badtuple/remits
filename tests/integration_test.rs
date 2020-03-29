/// All tests in this folder assume a server running on localhost:4242
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use serde::{Serialize};

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
async fn integration_tests() {
    let mut framer = connect_to_remits().await;
    should_respond_with!(framer, "LOG ADD test_log", b"+ok");

    // second create should be a noop, but still respond with "ok"
    should_respond_with!(framer, "LOG ADD test_log", b"+ok");

    // invalid iterator type
    let itr_cmd = "ITR ADD test_log test_itr_bad NOT_A_VALID_TYPE \n return msg";
    should_respond_with!(framer, itr_cmd, b"!ItrTypeInvalid");

    // create a valid iterator
    let itr_cmd = r"ITR ADD test_log test_itr MAP 
        return msg";
    should_respond_with!(framer, itr_cmd, b"+ok");

    // try to add invalid message pack
    let msg_cmd = b"MSG ADD test_log \x93\x00\x2a".to_vec();
    should_respond_with!(framer, msg_cmd, b"!MsgNotValidMessagePack");

    // add valid message
    #[derive(Serialize)]
    struct TestInput {
        name: String,
        number: usize,
        list: Vec<usize>,
    }

    let mp = rmp_serde::to_vec(&TestInput {
        name: "testing".to_owned(),
        number: 42,
        list: vec![1, 2, 3],
    })
    .unwrap();

    let mut msg_add_cmd = b"MSG ADD test_log ".to_vec();
    msg_add_cmd.append(&mut mp.clone());
    should_respond_with!(framer, msg_add_cmd, b"+ok");

    let itr_next_cmd = b"ITR NEXT test_itr 0 1".to_vec();
    should_respond_with!(framer, itr_next_cmd, b"+ok");
}
