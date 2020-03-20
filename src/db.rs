use std::collections::HashMap;

use crate::parser::Command;
use bytes::Bytes;

// Temporarily, everything will be done in memory until we're happy with the
// interface.
#[derive(Debug)]
pub struct DB {
    logs: HashMap<String, Vec<String>>,
}

impl DB {
    pub fn new() -> Self {
        DB {
            logs: HashMap::new(),
        }
    }

    pub fn exec(&mut self, cmd: Command) -> Result<String, Error> {
        use Command::*;

        match cmd {
            LogAdd(name) => self.log_add(&*name),
            MsgAdd { log, msg } => self.msg_add(&*log, &*msg),
            _ => unimplemented!(),
        }
    }

    /// Adds a new log to the DB
    fn log_add(&mut self, name: &str) -> Result<String, Error> {
        self.logs.entry(name.to_owned()).or_insert(vec![]);
        Ok("ok".to_owned())
    }

    /// Adds a new message to a log
    fn msg_add(&mut self, log: &str, msg: &str) -> Result<String, Error> {
        match self.logs.get_mut(log) {
            Some(l) => {
                l.push(msg.to_owned());
                Ok("ok".to_owned())
            },
            None => Err(Error::LogDoesNotExist),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    LogDoesNotExist,
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("{:?}", e).into()
    }
}
