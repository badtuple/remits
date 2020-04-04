mod iters;
mod logs;
mod manifest;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::commands;
use crate::commands::Command;
use crate::errors::Error;
use crate::protocol::Response;
use logs::Log;
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

    pub fn exec(&mut self, cmd: Command) -> Response {
        use Command::*;

        match cmd {
            LogShow(commands::LogShow { log_name }) => self.log_show(log_name),
            LogAdd(commands::LogAdd { log_name }) => self.log_add(log_name),
            LogDelete(commands::LogDelete { log_name }) => self.log_delete(log_name),
            LogList => self.log_list(),
            IteratorList(commands::IteratorList { log_name }) => self.itr_list(log_name),
            MessageAdd(commands::MessageAdd { log_name, message }) => {
                self.msg_add(log_name, message)
            }
            IteratorAdd(commands::IteratorAdd {
                log_name,
                iterator_name,
                iterator_kind,
                iterator_func,
            }) => self.itr_add(log_name, iterator_name, iterator_kind, iterator_func),
            //ItrDel { log, name } => self.itr_del(log, name),
            IteratorNext(commands::IteratorNext {
                iterator_name,
                message_id,
                count,
            }) => self.itr_next(iterator_name, message_id, count),
            IteratorDelete(commands::IteratorDelete {
                log_name,
                iterator_name,
            }) => self.itr_del(log_name, iterator_name),
        }
    }

    /// List all logs in db
    fn log_list(&mut self) -> Response {
        Response::Data(
            self.manifest
                .logs
                .keys()
                .map(|key| key.as_bytes().to_vec())
                .collect(),
        )
    }

    /// Displays information about a log
    fn log_show(&mut self, name: String) -> Response {
        let info = serde_cbor::to_vec(&self.manifest.logs[&name])
            .expect("could not serialize log registrant");
        Response::Data(vec![info])
    }

    /// Adds a new log to the DB
    fn log_add(&mut self, name: String) -> Response {
        self.manifest.add_log(name.clone());
        self.logs.entry(name).or_insert_with(Log::new);
        Response::Info("ok".as_bytes().to_vec())
    }

    /// Deletes a log from the DB
    fn log_delete(&mut self, name: String) -> Response {
        if let Entry::Occupied(l) = self.logs.entry(name.clone()) {
            l.remove_entry();
            self.manifest.del_log(name);
        };
        Response::Info("ok".as_bytes().to_vec())
    }

    /// Adds a new message to a log
    fn msg_add(&mut self, log: String, msg: Vec<u8>) -> Response {
        let l = self.logs.get_mut(&log);
        if l.is_none() {
            return Error::LogDoesNotExist.into();
        }

        match l.unwrap().add_msg(msg) {
            Ok(_) => Response::Info("ok".as_bytes().to_vec()),
            Err(e) => e.into(),
        }
    }

    /// List all itrs attached to a log
    fn itr_list(&mut self, name: Option<String>) -> Response {
        let itrs = &self.manifest.itrs;
        let out = match name {
            Some(name) => itrs
                .iter()
                .filter(|(_, itr)| itr.log == name)
                .map(|(_, x)| x.name.as_bytes().to_vec())
                .collect(),
            None => itrs.keys().map(|key| key.as_bytes().to_vec()).collect(),
        };
        Response::Data(out)
    }

    /// Adds a new unindexed iterator to a log
    fn itr_add(&mut self, log: String, name: String, kind: String, func: String) -> Response {
        match self.manifest.add_itr(log, name, kind, func) {
            Ok(_) => Response::Info("ok".as_bytes().to_vec()),
            Err(e) => e.into(),
        }
    }
    // Delets an unused unindexed iterator to a log
    fn itr_del(&mut self, log: String, name: String) -> Response {
        match self.manifest.del_itr(log, name) {
            Ok(_) => Response::Info("ok".as_bytes().to_vec()),
            Err(e) => e.into(),
        }
    }

    fn itr_next(&mut self, name: String, msg_id: usize, count: usize) -> Response {
        let itr = match self.manifest.itrs.get(&name) {
            Some(itr) => itr,
            None => return Error::ItrDoesNotExist.into(),
        };

        let log = match self.logs.get(&itr.log) {
            Some(log) => log,
            None => return Error::LogDoesNotExist.into(),
        };

        match itr.next(log, msg_id, count) {
            Ok(d) => Response::Data(d),
            Err(e) => e.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_log_list() {
        let mut db = DB::new();
        db.log_add("metric".to_owned());
        db.log_add("test".to_owned());

        let resp = db.log_list();
        match resp {
            Response::Data(bytes) => {
                let first = &*bytes[0];
                let secnd = &*bytes[1];
                assert!(first == &*"test".as_bytes() || first == &*"metric".as_bytes());
                assert!(secnd == &*"test".as_bytes() || secnd == &*"metric".as_bytes());
            }
            _ => panic!("error returned from log list"),
        };
    }

    #[test]
    fn test_db_log_show() {
        let mut db = DB::new();
        db.log_add("test".to_owned());
        let resp = db.log_show("test".to_owned());

        match resp {
            Response::Data(bytes) => assert_eq!(*bytes[0], *"test".as_bytes()),
            Response::Error(e) => panic!("error returned from log show: {:#?}", e),
            Response::Info(i) => panic!("info returned from log show: {:#?}", i),
        }
    }

    //#[test]
    //fn test_db_log_add() {
    //let mut db = DB::new();
    //let out = db.log_add("test".to_owned()).unwrap();
    //assert_eq!(out, "ok".to_owned());
    //let out2 = db.log_add("test".to_owned()).unwrap();
    //assert_eq!(out2, "ok".to_owned());
    //}

    //#[test]
    //fn test_db_msg_add() {
    //let mut db = DB::new();
    //let _ = db.log_add("test".to_owned()).unwrap();

    //Normal
    //let msg = vec![0x19, 0x03, 0xE8];
    //let out = db.msg_add("test".to_owned(), msg.clone()).unwrap();
    //assert_eq!(out, "ok".to_owned());
    //assert_eq!(db.logs.len(), 1);
    //assert_eq!(db.logs["test"][0], msg.clone());

    //let out = db.msg_add("test".to_owned(), msg.clone()).unwrap();
    //assert_eq!(out, "ok".to_owned());
    //assert_eq!(db.logs.len(), 1);
    //assert_eq!(db.logs["test"][1], msg.clone());
    //}

    //#[test]
    //#[should_panic]
    //fn test_db_msg_add_log_dne() {
    //let mut db = DB::new();
    //db.msg_add("test".to_owned(), "hello".as_bytes().to_vec())
    //.unwrap();
    //}

    //#[test]
    //fn test_db_log_del() {
    //let mut db = DB::new();
    //let _ = db.log_add("test".to_owned()).unwrap();
    //let _ = db.manifest.add_itr(
    //"test".to_owned(),
    //"fun".to_owned(),
    //"map".to_owned(),
    //"func".to_owned(),
    //);
    //assert_eq!(db.manifest.logs.len(), 1);
    //Normal
    //let out = db.log_del("test".to_owned()).unwrap();
    //assert_eq!(out, "ok".to_owned());
    //assert_eq!(db.manifest.logs.len(), 0);
    //Repeat
    //let out = db.log_del("test".to_owned()).unwrap();
    //assert_eq!(out, "ok".to_owned());
    //Never existed
    //let out = db.log_del("test1".to_owned()).unwrap();
    //assert_eq!(out, "ok".to_owned());
    //}

    //#[test]
    //fn test_db_itr_list() {
    //let mut db = DB::new();
    //let _ = db.log_add("test".to_owned()).unwrap();
    //let _ = db
    //.itr_add(
    //"test".to_owned(),
    //"std_dev avg users".to_owned(),
    //"map".to_owned(),
    //"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+."
    //.to_owned(),
    //)
    //.unwrap();
    //let _ = db
    //.itr_add(
    //"test2".to_owned(),
    //"std_dev avg users2".to_owned(),
    //"map".to_owned(),
    //"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+."
    //.to_owned(),
    //)
    //.unwrap();
    //let out = db.itr_list("test".to_owned()).unwrap();
    //assert!(out.contains("std_dev avg users"));
    //let out = db.itr_list("".to_owned()).unwrap();
    //assert!(out.contains("std_dev avg users2"));
    //}

    //This test is not needed now. It is a wrapper for manifest.add_iter()
    //#[test]
    //fn test_db_itr_add() {
    //let mut db = DB::new();
    //let out = db
    //.itr_add(
    //"test".to_owned(),
    //"std_dev avg users".to_owned(),
    //"map".to_owned(),
    //"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+."
    //.to_owned(),
    //)
    //.unwrap();
    //assert_eq!(out, "ok".to_owned());
    //assert_eq!(db.manifest.itrs.len(), 1);
    //}

    //This test is not needed now. It is a wrapper for manifest.del_iter()
    //#[test]
    //fn test_db_itr_del() {
    //let mut db = DB::new();
    //let _ = db.itr_add(
    //"test".to_owned(),
    //"std_dev avg users".to_owned(),
    //"map".to_owned(),
    //"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.".to_owned(),
    //);
    //let out = db
    //.itr_del("test".to_owned(), "std_dev avg users".to_owned())
    //.unwrap();
    //assert_eq!(out, "ok".to_owned());
    //assert_eq!(db.manifest.itrs.len(), 0);
    //}

    //This test is not needed now. if any logic other than match is added to exec we would add it here
    //#[test]
    //fn test_db_exec() {
    //let mut db = DB::new();
    //let out = db.exec(Command::LogAdd("test".to_owned())).unwrap();
    //assert_eq!(out, "ok");
    //}
}
