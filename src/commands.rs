use serde::Deserialize;

#[derive(Debug)]
pub enum Command {
    LogShow(LogShow),
    LogAdd(LogAdd),
    LogDelete(LogDelete),
    LogList,
    MessageAdd(MessageAdd),
    IteratorAdd(IteratorAdd),
    IteratorList(IteratorList),
    IteratorDelete(IteratorDelete),
    IteratorNext(IteratorNext),
}

#[derive(Deserialize, Debug)]
pub struct LogShow {
    pub log_name: String,
}

#[derive(Deserialize, Debug)]
pub struct LogAdd {
    pub log_name: String,
}

#[derive(Deserialize, Debug)]
pub struct LogDelete {
    pub log_name: String,
}

#[derive(Deserialize, Debug)]
pub struct MessageAdd {
    pub log_name: String,
    pub message: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct IteratorAdd {
    pub log_name: String,
    pub iterator_name: String,
    pub iterator_kind: IteratorKind,
    pub iterator_func: String,
}

#[derive(Deserialize, Debug)]
pub struct IteratorList {
    pub log_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct IteratorNext {
    pub iterator_name: String,
    pub message_id: usize,
    pub count: usize,
}

#[derive(Deserialize, Debug)]
pub struct IteratorDelete {
    pub log_name: String,
    pub iterator_name: String,
}

#[derive(PartialEq, Eq, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IteratorKind {
    Map,
    Filter,
    Reduce,
}

impl From<&str> for IteratorKind {
    fn from(s: &str) -> IteratorKind {
        match &*s.to_lowercase() {
            "map" => IteratorKind::Map,
            "filter" => IteratorKind::Filter,
            "reduce" => IteratorKind::Reduce,
            _ => panic!("unexpected iterator kind"),
        }
    }
}
