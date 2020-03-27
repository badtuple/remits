#[derive(Debug, PartialEq, Eq)]
pub struct Itr {
    pub log: String,
    pub name: String,
    pub func: String,
    pub kind: ItrKind,
}

impl Itr {}

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
