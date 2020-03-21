use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::SystemTime;

use crate::parser::Command;
use bytes::Bytes;

// Temporarily, everything will be done in memory until we're happy with the
// interface.
#[derive(Debug)]
pub struct DB {
    manifest: Manifest,
    logs: HashMap<String, Log>,
}

impl DB {
    pub fn new() -> Self {
        DB {
            manifest: Manifest::new(),
            logs: HashMap::new(),
        }
    }

    pub fn exec(&mut self, cmd: Command) -> Result<String, Error> {
        use Command::*;

        match cmd {
            LogAdd(name) => self.log_add(name),
            LogDel(name) => self.log_del(name),
            MsgAdd { log, msg } => self.msg_add(log, msg),
            ItrAdd {
                log,
                name,
                kind,
                func,
            } => self.itr_add(log, name, kind, func),
        }
    }

    /// Adds a new log to the DB
    fn log_add(&mut self, name: String) -> Result<String, Error> {
        self.manifest.add_log(name.clone());
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
            }
            None => Err(Error::LogDoesNotExist),
        }
    }

    /// Adds a new unindexed iterator to a log
    fn itr_add(
        &mut self,
        log: String,
        name: String,
        kind: String,
        func: String,
    ) -> Result<String, Error> {
        self.manifest.add_itr(log, name, kind, func)?;
        Ok("ok".to_owned())
    }
}

type Log = Vec<Vec<u8>>;

/// The Manifest is a file at the root of the database directory that is used
/// as a registry for database constructs such as Logs and Iters. It will map
/// the identifiers of those constructs to their corresponding files, along
/// with any metadata needed.
///
/// Right now the Manifest is held in memory, just like the rest of POC database
/// until we are happy with the interface.
#[derive(Debug)]
struct Manifest {
    /// List of all existing logs
    logs: HashMap<String, LogRegistrant>,

    /// List of all existing Iters
    /// TODO: Once Iters are built out, store the actual code so they can be
    /// rebuilt.  For now, it's just the identifier.
    itrs: HashMap<String, ItrRegistrant>,
}

impl Manifest {
    fn new() -> Self {
        Manifest {
            logs: HashMap::new(),
            itrs: HashMap::new(),
        }
    }

    fn add_log(&mut self, name: String) {
        self.logs
            .entry(name.clone())
            .or_insert_with(|| LogRegistrant {
                name: name,
                created_at: SystemTime::now(),
            });
    }

    fn add_itr(
        &mut self,
        log: String,
        name: String,
        kind: String,
        func: String,
    ) -> Result<(), Error> {
        let entry = self.itrs.entry(name.clone());
        match entry {
            Entry::Occupied(e) => {
                let itr = e.get();
                if itr.log != log || itr.kind != kind || itr.func != func {
                    return Err(Error::ItrExistsWithSameName);
                }
            }
            Entry::Vacant(e) => {
                e.insert(ItrRegistrant {
                    log,
                    name,
                    kind,
                    func,
                });
            }
        };

        Ok(())
    }
}

/// The Manifest entry for a Log
#[derive(Debug)]
struct LogRegistrant {
    name: String,
    created_at: SystemTime,
}

#[derive(Debug)]
struct ItrRegistrant {
    log: String,
    name: String,
    kind: String,
    func: String,
}

#[derive(Debug)]
pub enum Error {
    LogDoesNotExist,
    ItrExistsWithSameName,
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("{:?}", e).into()
    }
}
