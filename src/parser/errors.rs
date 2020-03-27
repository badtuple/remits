use bytes::Bytes;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    UnrecognizedCommand,
    MalformedCommand,
    NotEnoughArguments,

    LogNameNotUtf8,
    ItrNameNotUtf8,
    ItrTypeNotUtf8,
    ItrFuncNotUtf8,

    ItrTypeInvalid,
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("err {:?}", e).into()
    }
}
