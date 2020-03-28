use super::logs::Log;

#[derive(Debug, PartialEq, Eq)]
pub struct Itr {
    pub log: String,
    pub name: String,
    pub func: String,
    pub kind: ItrKind,
}

impl Itr {
    pub fn next(&self, log: &Log, offset: usize, count: usize) -> Vec<Vec<u8>> {
        // TODO: this will panic if count is out of bounds.
        // Implement `get` on Log and return None if nothing exists.
        (0..count).map(|i| log[offset + i].clone()).collect()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ItrKind {
    Map,
    Filter,
    Reduce,
}

impl PartialEq<String> for ItrKind {
    fn eq(&self, other: &String) -> bool {
        use ItrKind::*;
        let itr_kind = match &**other {
            "map" => Map,
            "filter" => Filter,
            "reduce" => Reduce,
            _ => return false,
        };

        *self == itr_kind
    }
}

pub fn string_to_kind_unchecked(s: String) -> ItrKind {
    use ItrKind::*;
    match &*s {
        "map" => Map,
        "filter" => Filter,
        "reduce" => Reduce,
        _ => panic!("string_to_kind_unchecked called with unvalidated string"),
    }
}
