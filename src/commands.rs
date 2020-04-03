use serde::Deserialize;

pub enum Command {
    LogShow(LogShow),
    LogAdd(LogAdd),
    LogDelete(LogDelete),
    LogList,
    MessageAdd(MessageAdd),
    IteratorAdd(IteratorAdd),
    IteratorList,
    IteratorNext,
}

#[derive(Deserialize)]
pub struct LogShow {
    pub log_name: String,
}

#[derive(Deserialize)]
pub struct LogAdd {
    pub log_name: String,
}

#[derive(Deserialize)]
pub struct LogDelete {
    pub log_name: String,
}

#[derive(Deserialize)]
pub struct MessageAdd {
    pub log_name: String,
    pub message: Vec<u8>,
}

#[derive(Deserialize)]
pub struct IteratorAdd {
    pub log_name: String,
    pub iterator_name: String,
    pub iterator_kind: String,
    pub iterator_func: String,
}
