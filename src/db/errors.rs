use bytes::Bytes;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    LogDoesNotExist,
    ItrExistsWithSameName,
    ItrDoesNotExist,
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("err {:?}", e).into()
    }
}
