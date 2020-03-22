mod errors;
mod manifest;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::parser::Command;
use errors::Error;
use manifest::Manifest;

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

#[cfg(test)]
mod tests {
    use super::*;
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
