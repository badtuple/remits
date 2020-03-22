#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    LogAdd(String),
    LogDel(String),
    MsgAdd {
        log: String,
        msg: Vec<u8>,
    },
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
}

impl Command {
    pub fn new_itr_add(log: &str, name: &str, kind: &str, func: &str) -> Self {
        Self::ItrAdd {
            log: log.to_owned(),
            name: name.to_owned(),
            kind: kind.to_owned(),
            func: func.to_owned(),
        }
    }

    pub fn new_itr_del(log: &str, name: &str) -> Self {
        Self::ItrDel {
            log: log.to_owned(),
            name: name.to_owned(),
        }
    }
}
