use bytes::Bytes;
use std::str::from_utf8;

pub fn parse(input: &[u8]) -> Result<Command, Error> {
    // Right now, the first 7 charactes of any valid query designates the command.
    if input.len() < 7 {
        return Err(Error::MalformedCommand);
    }

    let (cmd_str, data) = input.split_at(8);
    match cmd_str {
        b"LOG ADD " => parse_log_add(data),
        b"LOG DEL " => parse_log_del(data),
        b"MSG ADD " => parse_msg_add(data),
        _ => {
            debug!("{:?}", cmd_str);
            Err(Error::UnrecognizedCommand)
        },
    }
}

fn parse_log_add(data: &[u8]) -> Result<Command, Error> {
    match from_utf8(data) {
        Ok(s) => Ok(Command::LogAdd(s.to_owned())),
        Err(_) => Err(Error::LogNameNotUtf8),
    }
}

fn parse_log_del(data: &[u8]) -> Result<Command, Error> {
    match from_utf8(data) {
        Ok(s) => Ok(Command::LogDel(s.to_owned())),
        Err(_) => Err(Error::LogNameNotUtf8),
    }
}

fn parse_msg_add(data: &[u8]) -> Result<Command, Error> {
    let parts: Vec<&[u8]> = data.splitn(2, |b| *b == b' ').collect();
    match &*parts {
        [log_u8, msg] => match from_utf8(log_u8) {
            Ok(log) => Ok(Command::MsgAdd {
                log: log.to_owned(),
                msg: msg.to_vec(),
            }),
            Err(_) => Err(Error::LogNameNotUtf8),
        },
        _ => Err(Error::NotEnoughArguments),
    }
}

pub enum Command {
    LogAdd(String),
    LogDel(String),
    MsgAdd { log: String, msg: Vec<u8> },
    ItrAdd(String),
}

#[derive(Debug)]
pub enum Error {
    UnrecognizedCommand,
    MalformedCommand,
    LogNameNotUtf8,
    NotEnoughArguments,
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("{:?}", e).into()
    }
}
