/// All tests in this folder assume a server running on localhost:4242
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use serde::{Deserialize, Serialize};

use bytes::Bytes;

static LOCAL_REMITS: &str = "localhost:4242";
static OK_RESP: &[u8] = &[0x62, 0x6F, 0x6B];

#[tokio::test]
async fn integration_tests() {
    let ref mut framer = connect_to_remits().await;

    println!("test: should be able to add a log");
    let (kind, code, payload) = send_req(framer, new_log_add_req("test")).await;
    assert_eq!(kind, 0x01);
    assert_eq!(code, 0x00);
    assert_eq!(payload, OK_RESP);

    println!("test: re-adding an existing log should be a noop and still return ok");
    let (kind, code, payload) = send_req(framer, new_log_add_req("test")).await;
    assert_eq!(kind, 0x01);
    assert_eq!(code, 0x00);
    assert_eq!(payload, OK_RESP);

    println!("test: invalid iterator types should return an error");
    let (kind, code, payload) =
        send_req(framer, new_itr_add_req("test", "itr", "NOT_A_VALID_TYPE")).await;
    assert_eq!(kind, 0x03);
    assert_eq!(code, 0x0B);
    assert_eq!(payload, serde_cbor::to_vec(&"CouldNotReadPayload").unwrap());

    println!("test: should be able to add an iterator");
    let (kind, code, payload) = send_req(framer, new_itr_add_req("test", "itr", "map")).await;
    assert_eq!(kind, 0x01);
    assert_eq!(code, 0x00);
    assert_eq!(payload, OK_RESP);

    println!("test: should not be able to send a message as invalid cbor");
    let (kind, code, payload) =
        send_req(framer, new_msg_add_req("test", b"\x93\x00\x2a".to_vec())).await;
    assert_eq!(kind, 0x03);
    assert_eq!(code, 0x03);
    assert_eq!(payload, serde_cbor::to_vec(&"MsgNotValidCbor").unwrap());

    println!("test: can add valid message");

    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Msg {
        number: usize,
        word: String,
    }
    let test_msg = Msg {
        number: 42,
        word: "wat".into(),
    };

    let cbor = serde_cbor::to_vec(&test_msg).unwrap();
    println!("{:?}", cbor);

    let (kind, code, payload) = send_req(framer, new_msg_add_req("test", cbor.clone())).await;
    assert_eq!(kind, 0x01);
    assert_eq!(code, 0x00);
    assert_eq!(payload, OK_RESP);

    let (kind, code, payload) = send_req(framer, new_itr_next_req("itr", 0, 1)).await;
    assert_eq!(kind, 0x02);
    assert_eq!(code, 0x00);

    // Remove byte length
    let mut msg = &payload[4..];

    let resp: Msg = serde_cbor::from_reader(&mut msg).unwrap();
    assert_eq!(resp, test_msg);

    let (kind, code, payload) = send_req(framer, new_log_list_req()).await;
    assert_eq!(kind, 0x02);
    assert_eq!(code, 0x00);
    let out: Vec<String> = serde_cbor::from_slice(&payload[4..]).unwrap();
    assert_eq!(out, vec!("test"));
}

fn new_log_add_req(name: &str) -> Vec<u8> {
    #[derive(Serialize)]
    struct Body {
        log_name: String,
    }

    let mut body = vec![0x00, 0x01];
    let req = serde_cbor::to_vec(&Body {
        log_name: name.into(),
    })
    .unwrap();
    body.extend(req);
    body
}

fn new_itr_add_req(name: &str, itr_name: &str, typ: &str) -> Vec<u8> {
    #[derive(Serialize)]
    struct Body {
        log_name: String,
        iterator_name: String,
        iterator_kind: String,
        iterator_func: String,
    }

    let mut body = vec![0x00, 0x05];
    let req = serde_cbor::to_vec(&Body {
        log_name: name.into(),
        iterator_name: itr_name.into(),
        iterator_kind: typ.into(),
        iterator_func: "return msg".into(),
    })
    .unwrap();
    body.extend(req);
    body
}

fn new_msg_add_req(name: &str, message: Vec<u8>) -> Vec<u8> {
    #[derive(Serialize)]
    struct Body {
        log_name: String,
        message: Vec<u8>,
    }

    let mut body = vec![0x00, 0x04];
    let req = serde_cbor::to_vec(&Body {
        log_name: name.into(),
        message,
    })
    .unwrap();
    body.extend(req);
    body
}

fn new_log_list_req() -> Vec<u8> {
    vec![0x00, 0x03]
}

fn new_itr_next_req(name: &str, message_id: usize, count: usize) -> Vec<u8> {
    #[derive(Serialize)]
    struct Body {
        iterator_name: String,
        message_id: usize,
        count: usize,
    }

    let mut body = vec![0x00, 0x07];
    let req = serde_cbor::to_vec(&Body {
        iterator_name: name.into(),
        message_id,
        count,
    })
    .unwrap();
    body.extend(req);
    body
}

// returns Kind, Code, and Payload
async fn send_req(
    framer: &mut Framed<TcpStream, LengthDelimitedCodec>,
    bytes: Vec<u8>,
) -> (u8, u8, Vec<u8>) {
    framer
        .send(Bytes::from(bytes))
        .await
        .expect("could not send command");

    let result = framer
        .next()
        .await
        .expect("no response from remits")
        .expect("could not understand response");

    return (result[0], result[1], result[2..].to_vec());
}

async fn connect_to_remits() -> Framed<TcpStream, LengthDelimitedCodec> {
    let stream = TcpStream::connect(LOCAL_REMITS)
        .await
        .expect("could not connect to localhost:4242");

    Framed::new(stream, LengthDelimitedCodec::new())
}
