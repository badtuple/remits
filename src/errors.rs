use bytes::Bytes;

#[derive(Debug, PartialEq, Eq, ToPrimitive)]
pub enum Error {
    // DB Errors
    LogDoesNotExist,
    ItrExistsWithSameName,
    ItrDoesNotExist,
    MsgNotValidCbor,
    InvalidMsgPack,
    ErrRunningLua,
    ErrReadingLuaResponse,

    // Protocol Errors
    ConnectionClosed,
    UnknownRequestCode,
    UnknownFrameKind,
    FailedToReadBytes,
    ServerOnlyAcceptsRequests,
    CouldNotReadPayload,

    // Request Validations
    LogNameNotUtf8,
    ItrNameNotUtf8,
    ItrTypeNotUtf8,
    ItrFuncNotUtf8,
    ItrTypeInvalid,
    MsgIdNotNumber,
}

impl Error {
    pub fn to_bytes(self) -> Vec<u8> {
        format!("{:?}", self).into()
    }
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("{:?}", e).into()
    }
}
