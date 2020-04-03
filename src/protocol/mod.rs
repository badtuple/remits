use crate::commands::Command;
use crate::errors::Error;
use bytes::{Bytes, BytesMut};
use futures::SinkExt;
use num_traits::{FromPrimitive, ToPrimitive};
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(FromPrimitive, ToPrimitive, PartialEq, Eq)]
enum FrameKind {
    Request = 0x00,
    Info = 0x01,
    Data = 0x02,
    Error = 0x03,
}

#[derive(FromPrimitive, ToPrimitive)]
enum RequestCodes {
    LogShow = 0x00,
    LogAdd = 0x01,
    LogDelete = 0x02,
    LogList = 0x03,
    MessageAdd = 0x04,
    IteratorAdd = 0x05,
    IteratorList = 0x06,
    IteratorNext = 0x07,
}

pub struct Connection {
    framer: Framed<TcpStream, LengthDelimitedCodec>,
}

impl Connection {
    pub async fn next_request(&mut self) -> Option<Result<Command, Error>> {
        let frame = match self.framer.next().await {
            Some(f) => f,
            None => return None,
        };

        let result = match frame {
            Ok(bytes) => read_command(bytes),
            Err(_) => Err(Error::FailedToReadBytes),
        };

        Some(result)
    }

    pub async fn respond(&mut self, resp: Response) {
        debug!("responding with: {:?}", &resp);
        if let Err(e) = self.framer.send(resp.into()).await {
            error!("could not respond: {}", e);
        };
    }
}

impl From<TcpStream> for Connection {
    fn from(socket: TcpStream) -> Connection {
        let mut framer = Framed::new(socket, LengthDelimitedCodec::new());
        Connection { framer }
    }
}

fn read_command(bytes: BytesMut) -> Result<Command, Error> {
    let kind = match FrameKind::from_u8(bytes[0]) {
        Some(k) => k,
        None => return Err(Error::UnknownFrameKind),
    };

    let code = match RequestCodes::from_u8(bytes[1]) {
        Some(c) => c,
        None => return Err(Error::UnknownRequestCode),
    };

    if kind != FrameKind::Request {
        return Err(Error::ServerOnlyAcceptsRequests);
    }

    let data = &bytes[2..];

    use RequestCodes::*;
    let cmd = match code {
        LogShow => {
            let c = match serde_cbor::from_slice(data) {
                Ok(c) => c,
                Err(_) => return Err(Error::CouldNotReadPayload),
            };
            Command::LogShow(c)
        }
        _ => unimplemented!(),
    };

    Ok(cmd)
}

#[derive(Debug)]
pub enum Response {
    Info(Vec<u8>),
    Data(Vec<Vec<u8>>),
    Error(Error),
}

impl From<Error> for Response {
    fn from(e: Error) -> Response {
        Response::Error(e)
    }
}

impl From<Response> for Bytes {
    fn from(r: Response) -> Bytes {
        match r {
            Response::Info(data) => [&[FrameKind::Info.to_u8().unwrap(), 0x00 as u8], &*data]
                .concat()
                .into(),
            Response::Data(datas) => {
                let mut byt = vec![FrameKind::Data.to_u8().unwrap(), 0x00 as u8];
                for mut data in datas {
                    let len = u32::to_be_bytes(data.len() as u32);
                    byt.extend_from_slice(&len);
                    byt.append(&mut data);
                }
                byt.into()
            }
            _ => unimplemented!(),
        }
    }
}
