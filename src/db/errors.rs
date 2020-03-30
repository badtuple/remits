use bytes::Bytes;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    LogDoesNotExist,
    ItrExistsWithSameName,
    ItrDoesNotExist,
    MsgNotValidCbor,
    InvalidMsgPack,
    ErrRunningLua,
    ErrReadingLuaResponse,
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
