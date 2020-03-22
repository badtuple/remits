use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::SystemTime;

use crate::parser::Command;
use bytes::Bytes;

// Temporarily, everything will be done in memory until we're happy with the
// interface.
#[derive(Debug, PartialEq, Eq)]
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
            ItrDel { log, name } => self.itr_del(log, name),
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
    // Delets an unused unindexed iterator to a log
    fn itr_del(&mut self, log: String, name: String) -> Result<String, Error> {
        self.manifest.del_itr(log, name)?;
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
#[derive(Debug, PartialEq, Eq)]
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
                name,
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

    fn del_itr(&mut self, log: String, name: String) -> Result<(), Error> {
        let entry = self.itrs.entry(name);
        match entry {
            Entry::Occupied(e) => {
                let itr = e.get();
                if itr.log != log {
                    return Err(Error::ItrDoesNotExist);
                }
                let _ = e.remove();
            }
            Entry::Vacant(_e) => {
                return Err(Error::ItrDoesNotExist);
            }
        };

        Ok(())
    }
}

/// The Manifest entry for a Log
#[derive(Debug, PartialEq, Eq)]
struct LogRegistrant {
    name: String,
    created_at: SystemTime,
}

#[derive(Debug, PartialEq, Eq)]
struct ItrRegistrant {
    log: String,
    name: String,
    kind: String,
    func: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    LogDoesNotExist,
    ItrExistsWithSameName,
    ItrDoesNotExist,
}

impl From<Error> for Bytes {
    fn from(e: Error) -> Self {
        format!("{:?}", e).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_manifest_new() {
        let manifest = Manifest::new();
        assert_eq!(
            manifest,
            Manifest {
                logs: HashMap::new(),
                itrs: HashMap::new(),
            }
        );
    }
    #[test]
    fn test_manifest_add_log() {
        let mut manifest = Manifest::new();
        manifest.add_log("test".to_owned());
        manifest.add_log("test2".to_owned());
        manifest.add_log("test3".to_owned());
        assert!(manifest.logs.contains_key("test"));
        assert!(manifest.logs.contains_key("test2"));
        assert!(manifest.logs.contains_key("test3"));
        assert_eq!(manifest.logs.contains_key("test1"), false);

        // This second add_log is here to make sure code does not panic
        manifest.add_log("test".to_owned());
    }
    #[test]
    fn test_manifest_add_itr() {
        let mut manifest = Manifest::new();
        let _ = manifest.add_itr(
            "test".to_owned(),
            "fun".to_owned(),
            "lua".to_owned(),
            "func".to_owned(),
        );
        let _ = manifest.add_itr(
            "test".to_owned(),
            "fun2".to_owned(),
            "lua".to_owned(),
            "func".to_owned(),
        );
        let _ = manifest.add_itr(
            "test".to_owned(),
            "fun3".to_owned(),
            "lua".to_owned(),
            "func".to_owned(),
        );
        assert!(manifest.itrs.contains_key("fun"));
        assert!(manifest.itrs.contains_key("fun2"));
        assert!(manifest.itrs.contains_key("fun3"));
        assert_eq!(manifest.logs.contains_key("fun1"), false);

        let duplicate_error = manifest.add_itr(
            "test".to_owned(),
            "fun".to_owned(),
            "lua".to_owned(),
            "func2".to_owned(),
        );
        assert_eq!(
            format!("{:?}", duplicate_error),
            format!("Err(ItrExistsWithSameName)")
        );
    }

    #[test]
    fn test_manifest_del_itr() {
        let mut manifest = Manifest::new();
        // Normal
        let _ = manifest.add_itr(
            "test".to_owned(),
            "fun".to_owned(),
            "lua".to_owned(),
            "func".to_owned(),
        );
        assert!(manifest.itrs.contains_key("fun"));
        let _ = manifest.del_itr("test".to_owned(), "fun".to_owned());
        assert_eq!(manifest.logs.contains_key("fun"), false);

        // Function doesnt exist log does
        let does_not_exist_error = manifest.del_itr("test".to_owned(), "fun".to_owned());
        assert_eq!(
            format!("{:?}", does_not_exist_error),
            format!("Err(ItrDoesNotExist)")
        );
        // Neither function or log exist
        let _ = manifest.add_itr(
            "test".to_owned(),
            "fun".to_owned(),
            "lua".to_owned(),
            "func".to_owned(),
        );

        let log_does_not_exist_error = manifest.del_itr("test1".to_owned(), "fun".to_owned());
        assert_eq!(
            format!("{:?}", log_does_not_exist_error),
            format!("Err(ItrDoesNotExist)")
        );
    }

    #[test]
    fn test_db_new() {
        let db = DB::new();
        assert_eq!(
            db,
            DB {
                manifest: Manifest {
                    logs: HashMap::new(),
                    itrs: HashMap::new(),
                },
                logs: HashMap::new()
            }
        );
    }
    #[test]
    fn test_db_log_add() {
        let mut db = DB::new();
        let out = db.log_add("test".to_owned()).unwrap();
        assert_eq!(out, "ok".to_owned());
        let out2 = db.log_add("test".to_owned()).unwrap();
        assert_eq!(out2, "ok".to_owned());
    }
    #[test]
    fn test_db_log_del() {
        let mut db = DB::new();
        let _ = db.log_add("test".to_owned()).unwrap();
        // Normal
        let out = db.log_del("test".to_owned()).unwrap();
        assert_eq!(out, "ok".to_owned());
        // Repeat
        let out = db.log_del("test".to_owned()).unwrap();
        assert_eq!(out, "ok".to_owned());
        // Never existed
        let out = db.log_del("test1".to_owned()).unwrap();
        assert_eq!(out, "ok".to_owned());
    }
    #[test]
    fn test_db_msg_add() {
        let mut db = DB::new();
        let _ = db.log_add("test".to_owned()).unwrap();
        // Normal
        let out = db
            .msg_add("test".to_owned(), "hello".as_bytes().to_vec())
            .unwrap();
        assert_eq!(out, "ok".to_owned());
        assert_eq!(db.logs.len(), 1);
        assert_eq!(db.logs["test"][0], "hello".as_bytes().to_vec());

        let out = db
            .msg_add("test".to_owned(), "hello2".as_bytes().to_vec())
            .unwrap();
        assert_eq!(out, "ok".to_owned());
        assert_eq!(db.logs.len(), 1);
        assert_eq!(db.logs["test"][1], "hello2".as_bytes().to_vec());
    }
    #[test]
    #[should_panic]
    fn test_db_msg_add_log_dne() {
        let mut db = DB::new();
        db.msg_add("test".to_owned(), "hello".as_bytes().to_vec())
            .unwrap();
    }
    // This test is not needed now. It is a wrapper for manifest.add_iter()
    #[test]
    fn test_db_itr_add() {
        let mut db = DB::new();
        let out = db
            .itr_add(
                "test".to_owned(),
                "std_dev avg users".to_owned(),
                "bf".to_owned(),
                "+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+."
                    .to_owned(),
            )
            .unwrap();
        assert_eq!(out, "ok".to_owned());
        assert_eq!(db.manifest.itrs.len(), 1);
    }
    // This test is not needed now. It is a wrapper for manifest.del_iter()
    #[test]
    fn test_db_itr_del() {
        let mut db = DB::new();
        let _ = db.itr_add(
            "test".to_owned(),
            "std_dev avg users".to_owned(),
            "bf".to_owned(),
            "+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.".to_owned(),
        );
        let out = db
            .itr_del("test".to_owned(), "std_dev avg users".to_owned())
            .unwrap();
        assert_eq!(out, "ok".to_owned());
        assert_eq!(db.manifest.itrs.len(), 0);
    }
    // This test is not needed now. if any logic other than match is added to exec we would add it here
    #[test]
    fn test_db_exec() {
        let mut db = DB::new();
        let out = db.exec(Command::LogAdd("test".to_owned())).unwrap();
        assert_eq!(out, "ok");
    }
}
