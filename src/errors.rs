use bytes::Bytes;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, ToPrimitive, Serialize)]
pub enum Error {
    // DB Errors
    LogDoesNotExist = 0x00,
    ItrExistsWithSameName = 0x01,
    ItrDoesNotExist = 0x02,
    MsgNotValidCbor = 0x03,
    ErrRunningLua = 0x04,
    ErrReadingLuaResponse = 0x05,

    // Protocol Errors
    ConnectionClosed = 0x06,
    UnknownRequestCode = 0x07,
    UnknownFrameKind = 0x08,
    FailedToReadBytes = 0x09,
    ServerOnlyAcceptsRequests = 0x0A,
    CouldNotReadPayload = 0x0B,

    // Request Validations
    LogNameNotUtf8 = 0x0C,
    ItrNameNotUtf8 = 0x0D,
    ItrTypeNotUtf8 = 0x0E,
    ItrFuncNotUtf8 = 0x0F,
    ItrTypeInvalid = 0x10,
    MsgIdNotNumber = 0x11,
    MsgFieldNotOfTypeBinary = 0x12,
}

impl Error {
    pub fn to_bytes(self) -> Vec<u8> {
        serde_cbor::to_vec(&self).unwrap()
    }
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("{:?}", e).into()
    }
}
