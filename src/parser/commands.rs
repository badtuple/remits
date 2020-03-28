#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    LogShow(String),
    LogAdd(String),
    LogDel(String),
    LogList(),
    MsgAdd {
        log: String,
        msg: Vec<u8>,
    },
    ItrList(String),
    ItrAdd {
        log: String,
        name: String,
        kind: String,
        func: String,
    },
    ItrDel {
        log: String,
        name: String,
    },
    ItrNext {
        name: String,
        msg_id: usize,
        count: usize,
    },
}

impl Command {
    #[cfg(test)] // Adding this to remove lint warnings. Currently only used in tests
    pub fn new_itr_add(log: &str, name: &str, kind: &str, func: &str) -> Self {
        Self::ItrAdd {
            log: log.to_owned(),
            name: name.to_owned(),
            kind: kind.to_owned(),
            func: func.to_owned(),
        }
    }
    #[cfg(test)] // Adding this to remove lint warnings. Currently only used in tests
    pub fn new_itr_del(log: &str, name: &str) -> Self {
        Self::ItrDel {
            log: log.to_owned(),
            name: name.to_owned(),
        }
    }
}
