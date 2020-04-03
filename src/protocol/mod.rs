use bytes::{Bytes, BytesMut};
use futures::SinkExt;
use serde::Deserialize;
use std::io::Error;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(FromPrimitive, ToPrimitive)]
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

enum Request {
    LogShow,
    LogAdd,
    LogDelete,
    LogList,
    MessageAdd,
    IteratorAdd,
    IteratorList,
    IteratorNext,
}

impl From<BytesMut> for Request {
    fn from(self) -> Request {
        let kind = self[0];
        let code = self[1];

        use RequestCodes::*;
        match code {

        }
    }
}

#[derive(FromPrimitive, ToPrimitive)]
pub enum ErrorCodes {}

#[derive(Deserialize)]
struct LogShow {
    log_name: String,
}

pub struct Connection {
    framer: Framed<TcpStream, LengthDelimitedCodec>,
}

impl Connection {
    pub async fn next_request(&mut self) -> Option<Result<BytesMut, Error>> {
        let frame = self.framer.next().await;
        debug!("received command: {:?}", &frame);
        // TODO: use frame.map_ok() to convert BytesMut into an actual query.
        // Then return that Query.
        frame
    }

    pub async fn respond<T>(&mut self, resp: T)
    where
        T: Into<Bytes> + std::fmt::Debug,
    {
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
