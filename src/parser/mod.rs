mod commands;
mod errors;

pub use commands::Command;
pub use errors::Error;
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
        b"ITR ADD " => parse_itr_add(data),
        b"ITR DEL " => parse_itr_del(data),
        b"LOG LIST" => Ok(Command::LogList()),
        _ => {
            debug!("{:?}", cmd_str);
            Err(Error::UnrecognizedCommand)
        }
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

fn parse_itr_add(data: &[u8]) -> Result<Command, Error> {
    let parts: Vec<&[u8]> = data.splitn(4, |b| *b == b' ').collect();
    match &*parts {
        [raw_log, raw_itr, raw_kind, raw_func] => {
            let log = match from_utf8(raw_log) {
                Ok(log) => log.to_owned(),
                Err(_) => return Err(Error::LogNameNotUtf8),
            };

            let name = match from_utf8(raw_itr) {
                Ok(itr) => itr.to_owned(),
                Err(_) => return Err(Error::ItrNameNotUtf8),
            };

            let kind = match from_utf8(raw_kind) {
                Ok(kind) => kind.to_owned(),
                Err(_) => return Err(Error::ItrTypeNotUtf8),
            };

            let func = match from_utf8(raw_func) {
                Ok(f) => f.to_owned(),
                Err(_) => return Err(Error::ItrFuncNotUtf8),
            };

            Ok(Command::ItrAdd {
                log,
                name,
                kind,
                func,
            })
        }
        _ => Err(Error::NotEnoughArguments),
    }
}
fn parse_itr_del(data: &[u8]) -> Result<Command, Error> {
    let parts: Vec<&[u8]> = data.splitn(2, |b| *b == b' ').collect();
    match &*parts {
        [raw_log, raw_itr] => {
            let log = match from_utf8(raw_log) {
                Ok(log) => log.to_owned(),
                Err(_) => return Err(Error::LogNameNotUtf8),
            };

            let name = match from_utf8(raw_itr) {
                Ok(itr) => itr.to_owned(),
                Err(_) => return Err(Error::ItrNameNotUtf8),
            };

            Ok(Command::ItrDel { log, name })
        }
        _ => Err(Error::NotEnoughArguments),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commands::Command::*;
    use errors::Error::*;

    #[test]
    fn test_parse() {
        let unknown_command = parse("unknown command".as_bytes());
        assert_eq!(unknown_command, Err(UnrecognizedCommand));

        let undersized_command = parse("2short".as_bytes());
        assert_eq!(undersized_command, Err(MalformedCommand));

        let log_add_out = parse("LOG LIST".as_bytes());
        assert_eq!(log_add_out, Ok(LogList()));

        let log_add_out = parse("LOG ADD test".as_bytes());
        assert_eq!(log_add_out, Ok(LogAdd("test".to_owned())));

        let log_del_out = parse("LOG DEL test".as_bytes());
        assert_eq!(log_del_out, Ok(LogDel("test".to_owned())));

        let msg_add_out = parse("MSG ADD test testing comment".as_bytes());
        assert_eq!(
            msg_add_out,
            Ok(MsgAdd {
                log: "test".to_owned(),
                msg: b"testing comment".to_vec(),
            })
        );

        let itr_add_out = parse("ITR ADD test itr reduce func".as_bytes());
        assert_eq!(
            itr_add_out,
            Ok(Command::new_itr_add("test", "itr", "reduce", "func"))
        );

        let itr_del_out = parse("ITR DEL test itr".as_bytes());
        assert_eq!(itr_del_out, Ok(Command::new_itr_del("test", "itr")));
    }
}
