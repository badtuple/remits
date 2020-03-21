use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::parser::Command;
use bytes::Bytes;

// Temporarily, everything will be done in memory until we're happy with the
// interface.
#[derive(Debug)]
pub struct DB {
    logs: HashMap<String, Log>,
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
            LogAdd(name) => self.log_add(name),
            LogDel(name) => self.log_del(name),
            MsgAdd { log, msg } => self.msg_add(log, msg),
            _ => unimplemented!(),
        }
    }

    /// Adds a new log to the DB
    fn log_add(&mut self, name: String) -> Result<String, Error> {
        self.logs.entry(name).or_insert_with(|| vec![]);
        Ok("ok".to_owned())
    }

    /// Deletes a log from the DB
    fn log_del(&mut self, name: String) -> Result<String, Error> {
        if let Entry::Occupied(l) = self.logs.entry(name) {
            if l.get().is_empty() {
                l.remove_entry();
            }
        };

        Ok("ok".to_owned())
    }

    /// Adds a new message to a log
    fn msg_add(&mut self, log: String, msg: Vec<u8>) -> Result<String, Error> {
        match self.logs.get_mut(&log) {
            Some(l) => {
                l.push(msg);
                Ok("ok".to_owned())
            },
            None => Err(Error::LogDoesNotExist),
        }
    }
}

type Log = Vec<Vec<u8>>;

#[derive(Debug)]
pub enum Error {
    LogDoesNotExist,
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("{:?}", e).into()
    }
}
