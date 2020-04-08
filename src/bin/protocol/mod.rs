use std::io::{Read, Write};
use std::net::TcpStream;

use serde::Serialize;

pub static OK_RESP: &[u8] = &[0x62, 0x6F, 0x6B];

pub fn new_log_add_req(name: &str) -> Vec<u8> {
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
    let mut size = body.len().to_be_bytes().to_vec().to_vec();
    size.extend(body);
    size
}

pub fn new_log_show_req(name: &str) -> Vec<u8> {
    #[derive(Serialize)]
    struct Body {
        log_name: String,
    }

    let mut body = vec![0x00, 0x00];
    let req = serde_cbor::to_vec(&Body {
        log_name: name.into(),
    })
    .unwrap();
    body.extend(req);
    let mut size = body.len().to_be_bytes().to_vec();
    size.extend(body);
    size
}

pub fn new_log_del_req(name: &str) -> Vec<u8> {
    #[derive(Serialize)]
    struct Body {
        log_name: String,
    }

    let mut body = vec![0x00, 0x02];
    let req = serde_cbor::to_vec(&Body {
        log_name: name.into(),
    })
    .unwrap();
    body.extend(req);
    let mut size = body.len().to_be_bytes().to_vec();
    size.extend(body);
    size
}

pub fn new_iterator_add_req(name: &str, iterator_name: &str, typ: &str) -> Vec<u8> {
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
        iterator_name: iterator_name.into(),
        iterator_kind: typ.into(),
        iterator_func: "return msg".into(),
    })
    .unwrap();
    body.extend(req);
    let mut size = body.len().to_be_bytes().to_vec();
    size.extend(body);
    size
}

pub fn new_msg_add_req(name: &str, message: Vec<u8>) -> Vec<u8> {
    #[derive(Serialize)]
    struct Body {
        log_name: String,
        message: serde_cbor::Value,
    }

    let mut body = vec![0x00, 0x04];
    let req = serde_cbor::to_vec(&Body {
        log_name: name.into(),
        message: serde_cbor::Value::Bytes(message),
    })
    .unwrap();
    body.extend(req);
    let mut size = body.len().to_be_bytes().to_vec();
    size.extend(body);
    size
}

pub fn new_log_list_req() -> Vec<u8> {
    let body = vec![0x00, 0x03];
    let mut size = body.len().to_be_bytes().to_vec();
    size.extend(body);
    size
}
pub fn new_iterator_list_req() -> Vec<u8> {
    let body = vec![0x00, 0x06];
    let mut size = body.len().to_be_bytes().to_vec();
    size.extend(body);
    size
}
pub fn new_iterator_next_req(name: &str, message_id: usize, count: usize) -> Vec<u8> {
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
    let mut size = body.len().to_be_bytes().to_vec();
    size.extend(body);
    size
}

pub fn send_req(bytes: Vec<u8>) -> (u8, u8, Vec<u8>) {
    let mut stream = connect_to_remits();
    stream.write_all(&bytes).expect("could not send command");

    let mut buffer = [0; 4];
    stream.read_exact(&mut buffer).unwrap();
    let size = u32::from_be_bytes(buffer);
    let mut output_buffer = vec![0 as u8; (size) as usize].as_slice().to_owned();
    stream.read_exact(&mut output_buffer).expect("peek failed");

    (
        output_buffer[0],
        output_buffer[1],
        output_buffer[2..].to_vec(),
    )
}

pub fn connect_to_remits() -> TcpStream {
    if let Ok(stream) = TcpStream::connect("localhost:4242") {
        println!("Connected to remits!");
        stream
    } else {
        println!("Couldn't connect to server...");
        panic!()
    }
}

