/// All tests in this folder assume a server running on localhost:4242
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use serde::{Deserialize, Serialize};

use bytes::Bytes;

static LOCAL_REMITS: &str = "localhost:4242";
static OK_RESP: &[u8] = &[0x62, 0x6F, 0x6B];
#[derive(Serialize, Debug)]
pub struct AddLogBody {
    pub log_name: String,
}

impl AddLogBody {
    pub fn new_log_add_req(self) -> Vec<u8> {
        let mut body = vec![0x00, 0x01];
        let req = serde_cbor::to_vec(&self).unwrap();
        body.extend(req);
        body
    }
    fn new(name: &str) -> Self {
        let mut body = vec![0x00, 0x01];
        AddLogBody {
            log_name: name.into(),
        }
    }
}

#[derive(Serialize)]
pub struct ItrAddBody {
    pub log_name: String,
    pub iterator_name: String,
    pub iterator_kind: String,
    pub iterator_func: String,
}
impl ItrAddBody {
    pub fn new_itr_add_req(self) -> Vec<u8> {
        let mut body = vec![0x00, 0x05];
        let req = serde_cbor::to_vec(&self).unwrap();
        body.extend(req);
        body
    }
}
#[derive(Serialize)]
pub struct MsgAddBody {
    pub log_name: String,
    pub message: Vec<u8>,
}
impl MsgAddBody {
    pub fn new_msg_add_req(name: &str, message: Vec<u8>) -> Vec<u8> {
        let mut body = vec![0x00, 0x04];
        let req = serde_cbor::to_vec(&MsgAddBody {
            log_name: name.into(),
            message,
        })
        .unwrap();
        body.extend(req);
        body
    }
}
#[derive(Serialize)]
pub struct ItrNextBody {
    iterator_name: String,
    message_id: usize,
    count: usize,
}
impl ItrNextBody {
    pub fn new_itr_next_req(name: &str, message_id: usize, count: usize) -> Vec<u8> {
        let mut body = vec![0x00, 0x07];
        let req = serde_cbor::to_vec(&ItrNextBody {
            iterator_name: name.into(),
            message_id,
            count,
        })
        .unwrap();
        body.extend(req);
        body
    }
}

pub fn new_log_list_req() -> Vec<u8> {
    vec![0x00, 0x03]
}

// returns Kind, Code, and Payload
pub async fn send_req(
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

pub async fn connect_to_remits() -> Framed<TcpStream, LengthDelimitedCodec> {
    let stream = TcpStream::connect(LOCAL_REMITS)
        .await
        .expect("could not connect to localhost:4242");

    Framed::new(stream, LengthDelimitedCodec::new())
}
