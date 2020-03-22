use bytes::Bytes;

#[derive(Debug)]
pub enum Error {
    UnrecognizedCommand,
    MalformedCommand,
    NotEnoughArguments,

    LogNameNotUtf8,
    ItrNameNotUtf8,
    ItrTypeNotUtf8,
    ItrFuncNotUtf8,
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("{:?}", e).into()
    }
}
