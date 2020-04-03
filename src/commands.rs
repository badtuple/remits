use serde::Deserialize;

pub enum Command {
    LogShow(LogShow),
    LogAdd,
    LogDelete,
    LogList,
    MessageAdd,
    IteratorAdd,
    IteratorList,
    IteratorNext,
}

#[derive(Deserialize)]
pub struct LogShow {
    pub log_name: String,
}
