use std::str::from_utf8;
use bytes::Bytes;

pub fn parse(s: &[u8]) -> Result<Command, Error> {
    // Right now, the goal is that there will always be a 3 letter command,
    // a space, and a 3 letter primary arg. The rest is optional.
    if s.len() < 7 {
        return Err(Error::MalformedCommand);
    }

    use Command::*;
    use Error::*;

    let cmd = match s {
        // ew. there has to be a better way to do this
        [b'L', b'O', b'G', b' ', b'A', b'D', b'D', rest @ ..] => match from_utf8(rest) {
            Ok(s) => LogAdd(s.to_owned()),
            Err(_) => return Err(LogNameNotUtf8),
        },
        _ => return Err(UnrecognizedCommand),
    };

    Ok(cmd)
}

pub enum Command {
    LogAdd(String),
    LogDel(String),
    MsgAdd { log: String, msg: String },
    ItrAdd(String),
}

#[derive(Debug)]
pub enum Error {
    UnrecognizedCommand,
    MalformedCommand,
    LogNameNotUtf8,
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("{:?}", e).into()
    }
}
