#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    LogDoesNotExist,
    ItrExistsWithSameName,
    ItrDoesNotExist,
}

impl From<Error> for &[u8] {
    fn from(e: Error) -> Self {
        format!("err {:?}", e).as_bytes().clone()
    }
}
