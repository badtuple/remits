mod iters;
mod logs;
mod manifest;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::commands;
use crate::commands::{Command, IteratorKind};
use crate::errors::Error;
use crate::protocol::Response;
use logs::Log;
use manifest::Manifest;

const OK_RESP: &[u8] = &[0x62, 0x6F, 0x6B];

#[derive(Debug)]
pub struct DB {
    path: PathBuf,

    manifest: RwLock<Manifest>,
    logs: RwLock<HashMap<String, Log>>,
}

unsafe impl Send for DB {}
unsafe impl Sync for DB {}

impl DB {
    pub fn new(path: String) -> Self {
        let path = PathBuf::from(&*path);
        let mut manifest_path = path.clone();
        manifest_path.push("manifest");

        let manifest = if manifest_path.exists() {
            Manifest::load(&*manifest_path).expect("could not load manifest file")
        } else {
            Manifest::new(&*manifest_path)
        };

        DB {
            path,
            manifest: RwLock::new(manifest),
            logs: RwLock::new(HashMap::new()),
        }
    }

    pub fn exec(&self, cmd: Command) -> Response {
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
    fn log_list(&self) -> Response {
        let logs: Vec<String> = self
            .manifest
            .read()
            .expect("unwrapped poisoned manifest lock")
            .logs
            .keys()
            .cloned()
            .collect();
        let bytes = serde_cbor::to_vec(&logs).unwrap();
        Response::Data(vec![bytes])
    }

    /// Displays information about a log
    fn log_show(&self, name: String) -> Response {
        let m = self
            .manifest
            .read()
            .expect("unwrapped poisoned manifest lock");
        let info = serde_cbor::to_vec(&m.logs[&name]).expect("could not serialize log registrant");
        Response::Data(vec![info])
    }

    /// Adds a new log to the DB
    fn log_add(&self, name: String) -> Response {
        let mut m = self
            .manifest
            .write()
            .expect("unwrapped poisoned manifest lock");

        m.add_log(name.clone());

        self.logs
            .write()
            .expect("unwrapped poisoned logs lock")
            .entry(name.clone())
            .or_insert_with(|| Log::new(self.path.clone(), &*name));

        Response::Info(OK_RESP.into())
    }

    /// Deletes a log from the DB
    fn log_delete(&self, name: String) -> Response {
        let mut logs = self.logs.write().expect("unwrapped poisoned logs lock");

        if let Entry::Occupied(l) = logs.entry(name.clone()) {
            l.remove_entry();
            self.manifest
                .write()
                .expect("unwrapped poisoned manifest lock")
                .del_log(name);
        };
        Response::Info(OK_RESP.into())
    }

    /// Adds a new message to a log
    fn msg_add(&self, log: String, msg: Vec<u8>) -> Response {
        let mut logs = self.logs.write().expect("unwrapped poisoned logs lock");
        let l = logs.get_mut(&log);

        if l.is_none() {
            return Error::LogDoesNotExist.into();
        }

        match l.unwrap().add_msg(msg) {
            Ok(_) => Response::Info(OK_RESP.into()),
            Err(e) => e.into(),
        }
    }

    /// List all itrs attached to a log
    fn itr_list(&self, name: Option<String>) -> Response {
        let itrs = &self
            .manifest
            .read()
            .expect("unwrapped poisoned read lock")
            .itrs;

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
    fn itr_add(&self, log: String, name: String, kind: IteratorKind, func: String) -> Response {
        let mut m = self
            .manifest
            .write()
            .expect("unwrapped poisoned manifest lock");

        match m.add_itr(log, name, kind, func) {
            Ok(_) => Response::Info(OK_RESP.into()),
            Err(e) => e.into(),
        }
    }
    // Delets an unused unindexed iterator to a log
    fn itr_del(&self, log: String, name: String) -> Response {
        let mut m = self
            .manifest
            .write()
            .expect("unwrapped poisoned manifest lock");

        match m.del_itr(log, name) {
            Ok(_) => Response::Info(OK_RESP.into()),
            Err(e) => e.into(),
        }
    }

    fn itr_next(&self, name: String, msg_id: usize, count: usize) -> Response {
        let manifest = self
            .manifest
            .read()
            .expect("unwrapped poisoned manifest lock");
        let itr = match manifest.itrs.get(&name) {
            Some(itr) => itr,
            None => return Error::ItrDoesNotExist.into(),
        };

        let logs = self.logs.read().expect("unwrapped poisoned logs lock");
        let log = match logs.get(&itr.log) {
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
    use crate::test_util::temp_db_path;
    use std::time::SystemTime;

    #[test]
    fn test_db_log_list() {
        let db = DB::new(temp_db_path());
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
        let db = DB::new(temp_db_path());
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
        let db = DB::new(temp_db_path());

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
        let db = DB::new(temp_db_path());
        db.log_add("test".into());

        let msg = vec![0x19, 0x03, 0xE8];
        match db.msg_add("test".into(), msg.clone()) {
            Response::Info(i) => assert_eq!(i, OK_RESP),
            _ => panic!("expected info to be returned"),
        };

        let logs = db.logs.read().unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs["test"][0], msg);
    }

    #[test]
    fn test_db_msg_add_log_dne() {
        let db = DB::new(temp_db_path());
        match db.msg_add("test".into(), b"hello".to_vec()) {
            Response::Error(_e) => (),
            _ => panic!("expected response to be an error"),
        }
    }

    #[test]
    fn test_db_log_del() {
        let db = DB::new(temp_db_path());
        db.log_add("test".into());
        db.itr_add(
            "test".into(),
            "fun".into(),
            "map".into(),
            "return msg".into(),
        );
        assert_eq!(db.manifest.read().unwrap().logs.len(), 1);

        match db.log_delete("test".into()) {
            Response::Info(i) => assert_eq!(&*i, OK_RESP),
            _ => panic!("expected response to be info"),
        };
        assert_eq!(db.manifest.read().unwrap().logs.len(), 0);
    }

    #[test]
    fn test_db_itr_list() {
        let db = DB::new(temp_db_path());
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
                assert!((first == "i1" && secnd == "i2") || (secnd == "i1" && first == "i2"));
            }
            _ => panic!("expected itr_list to return data"),
        };
    }

    #[test]
    fn test_db_itr_add() {
        let db = DB::new(temp_db_path());
        match db.itr_add("log".into(), "i".into(), "map".into(), "return msg".into()) {
            Response::Info(i) => assert_eq!(i, OK_RESP),
            _ => panic!("expected itr_add to return info"),
        };
        assert_eq!(db.manifest.read().unwrap().itrs.len(), 1);
    }

    #[test]
    fn test_db_itr_del() {
        let db = DB::new(temp_db_path());
        db.itr_add("log".into(), "i".into(), "map".into(), "return msg".into());
        match db.itr_del("log".into(), "i".into()) {
            Response::Info(i) => assert_eq!(i, OK_RESP),
            _ => panic!("expected itr_add to return info"),
        };
        assert_eq!(db.manifest.read().unwrap().itrs.len(), 0);
    }
}
