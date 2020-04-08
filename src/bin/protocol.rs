use std::io::{Read, Write};
use std::net::TcpStream;

use serde::{Serialize};


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
    let mut size = transform_size_to_array_of_u8(body.len());
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
    let mut size = transform_size_to_array_of_u8(body.len());
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
    let mut size = transform_size_to_array_of_u8(body.len());
    size.extend(body);
    size
}

pub fn new_itr_add_req(name: &str, itr_name: &str, typ: &str) -> Vec<u8> {
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
    let mut size = transform_size_to_array_of_u8(body.len());
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
    let mut size = transform_size_to_array_of_u8(body.len());
    size.extend(body);
    size
}

pub fn new_log_list_req() -> Vec<u8> {
    let body = vec![0x00, 0x03];
    let mut size = transform_size_to_array_of_u8(body.len());
    size.extend(body);
    size
}
pub fn new_itr_list_req() -> Vec<u8> {
    let body = vec![0x00, 0x06];
    let mut size = transform_size_to_array_of_u8(body.len());
    size.extend(body);
    size
}
pub fn new_itr_next_req(name: &str, message_id: usize, count: usize) -> Vec<u8> {
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
    let mut size = transform_size_to_array_of_u8(body.len());
    size.extend(body);
    size
}

pub fn send_req(bytes: Vec<u8>) -> (u8, u8, Vec<u8>) {
    let mut stream = connect_to_remits();
    stream.write_all(&bytes).expect("could not send command");

    let mut buffer = [0; 4];
    stream.read_exact(&mut buffer).unwrap();
    let size = as_u32_be(buffer);
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

fn transform_size_to_array_of_u8(x: usize) -> Vec<u8> {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;
    [b1, b2, b3, b4].to_vec()
}

fn as_u32_be(array: [u8; 4]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + (array[3] as u32)
}
