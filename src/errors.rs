use bytes::Bytes;

#[derive(Debug, PartialEq, Eq)]
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

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("{:?}", e).into()
    }
}
impl From<Error> for Vec<u8> {
    fn from(e: Error) -> Self {
        let output: String = format!("{:?}", e).into();
        output.as_bytes().to_vec()
    }
}

//#[derive(FromPrimitive, ToPrimitive)]
//pub enum ErrorCodes {}
