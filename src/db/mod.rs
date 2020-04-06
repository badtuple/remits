mod iters;
mod logs;
mod manifest;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::commands;
use crate::commands::{Command, IteratorKind};
use crate::errors::Error;
use crate::protocol::Response;
use logs::Log;
use manifest::Manifest;

const OK_RESP: &[u8] = &[0x62, 0x6F, 0x6B];

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
            MessageAdd(commands::MessageAdd { log_name, message }) => match message {
                serde_cbor::Value::Bytes(m) => self.msg_add(log_name, m),
                _ => Error::MsgFieldNotOfTypeBinary.into(),
            },
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
        let logs: Vec<String> = self.manifest.logs.keys().cloned().collect();
        let bytes = serde_cbor::to_vec(&logs).unwrap();
        Response::Data(vec![bytes])
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
        Response::Info(OK_RESP.into())
    }

    /// Deletes a log from the DB
    fn log_delete(&mut self, name: String) -> Response {
        if let Entry::Occupied(l) = self.logs.entry(name.clone()) {
            l.remove_entry();
            self.manifest.del_log(name);
        };
        Response::Info(OK_RESP.into())
    }

    /// Adds a new message to a log
    fn msg_add(&mut self, log: String, msg: Vec<u8>) -> Response {
        let l = self.logs.get_mut(&log);
        if l.is_none() {
            return Error::LogDoesNotExist.into();
        }

        match l.unwrap().add_msg(msg) {
            Ok(_) => Response::Info(OK_RESP.into()),
            Err(e) => e.into(),
        }
    }

    /// List all itrs attached to a log
    fn itr_list(&mut self, name: Option<String>) -> Response {
        let itrs = &self.manifest.itrs;
        let out: Vec<Vec<u8>> = match name {
            Some(name) => itrs
                .iter()
                .filter(|(_, itr)| itr.log == name)
                .map(|(_, x)| serde_cbor::to_vec(&x.name).unwrap())
                .collect(),
            None => itrs
                .keys()
                .map(|x| serde_cbor::to_vec(x).unwrap())
                .collect(),
        };
        Response::Data(out)
    }

    /// Adds a new unindexed iterator to a log
    fn itr_add(&mut self, log: String, name: String, kind: IteratorKind, func: String) -> Response {
        match self.manifest.add_itr(log, name, kind, func) {
            Ok(_) => Response::Info(OK_RESP.into()),
            Err(e) => e.into(),
        }
    }
    // Delets an unused unindexed iterator to a log
    fn itr_del(&mut self, log: String, name: String) -> Response {
        match self.manifest.del_itr(log, name) {
            Ok(_) => Response::Info(OK_RESP.into()),
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
    use std::time::SystemTime;

    #[test]
    fn test_db_log_list() {
        let mut db = DB::new();
        db.log_add("metric".into());
        db.log_add("test".into());

        let resp = db.log_list();
        match resp {
            Response::Data(bytes) => {
                let out: Vec<String> = serde_cbor::from_slice(&*(bytes[0])).unwrap();
                let l1 = "test".to_owned();
                let l2 = "metric".to_owned();

                assert!(out[0] == l1 || out[1] == l1);
                assert!(out[1] == l2 || out[0] == l2);
            }
            _ => panic!("error returned from log list"),
        };
    }

    #[test]
    fn test_db_log_show() {
        let mut db = DB::new();
        db.log_add("test".into());
        let resp = db.log_show("test".into());

        let log = serde_cbor::to_vec(&manifest::LogRegistrant {
            name: "test".into(),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("could not get system time")
                .as_secs() as usize,
        })
        .expect("could not marshal comparison LogRegistrant");

        match resp {
            Response::Data(bytes) => assert_eq!(*bytes[0], *log),
            Response::Error(e) => panic!("error returned from log show: {:#?}", e),
            Response::Info(i) => panic!("info returned from log show: {:#?}", i),
        }
    }

    #[test]
    fn test_db_log_add() {
        let mut db = DB::new();

        match db.log_add("test".into()) {
            Response::Info(i) => assert_eq!(i, OK_RESP),
            _ => panic!("expected info to be returned"),
        };

        match db.log_add("test".into()) {
            Response::Info(i) => assert_eq!(i, OK_RESP),
            _ => panic!("expected info to be returned"),
        };
    }

    #[test]
    fn test_db_msg_add() {
        let mut db = DB::new();
        db.log_add("test".into());

        let msg = vec![0x19, 0x03, 0xE8];
        match db.msg_add("test".into(), msg.clone()) {
            Response::Info(i) => assert_eq!(i, OK_RESP),
            _ => panic!("expected info to be returned"),
        };

        assert_eq!(db.logs.len(), 1);
        assert_eq!(db.logs["test"][0], msg.clone());
    }

    #[test]
    fn test_db_msg_add_log_dne() {
        let mut db = DB::new();
        match db.msg_add("test".into(), "hello".as_bytes().to_vec()) {
            Response::Error(e) => (),
            _ => panic!("expected response to be an error"),
        }
    }

    #[test]
    fn test_db_log_del() {
        let mut db = DB::new();
        db.log_add("test".into());
        db.manifest.add_itr(
            "test".into(),
            "fun".into(),
            "map".into(),
            "return msg".into(),
        );
        assert_eq!(db.manifest.logs.len(), 1);

        match db.log_delete("test".into()) {
            Response::Info(i) => assert_eq!(&*i, OK_RESP),
            _ => panic!("expected response to be info"),
        };
        assert_eq!(db.manifest.logs.len(), 0);
    }

    #[test]
    fn test_db_itr_list() {
        let mut db = DB::new();
        db.log_add("log".into());
        db.itr_add("log".into(), "i1".into(), "map".into(), "return msg".into());
        db.itr_add(
            "log2".into(),
            "i2".into(),
            "map".into(),
            "return msg".into(),
        );
        match db.itr_list(Some("log".into())) {
            Response::Data(bytes) => {
                let out: String = serde_cbor::from_slice(&*(bytes[0])).unwrap();
                assert_eq!(out, "i1".to_owned());
            }
            _ => panic!("expected itr_list to return data"),
        };

        match db.itr_list(None) {
            Response::Data(bytes) => {
                let first: String = serde_cbor::from_slice(&*(bytes[0])).unwrap();
                let secnd: String = serde_cbor::from_slice(&*(bytes[1])).unwrap();
                assert!(
                    (first == "i1".to_owned() && secnd == "i2".to_owned())
                        || (secnd == "i1".to_owned() && first == "i2".to_owned())
                );
            }
            _ => panic!("expected itr_list to return data"),
        };
    }

    #[test]
    fn test_db_itr_add() {
        let mut db = DB::new();
        match db.itr_add("log".into(), "i".into(), "map".into(), "return msg".into()) {
            Response::Info(i) => assert_eq!(i, OK_RESP),
            _ => panic!("expected itr_add to return info"),
        };
        assert_eq!(db.manifest.itrs.len(), 1);
    }

    #[test]
    fn test_db_itr_del() {
        let mut db = DB::new();
        db.itr_add("log".into(), "i".into(), "map".into(), "return msg".into());
        match db.itr_del("log".into(), "i".into()) {
            Response::Info(i) => assert_eq!(i, OK_RESP),
            _ => panic!("expected itr_add to return info"),
        };
        assert_eq!(db.manifest.itrs.len(), 0);
    }
}
