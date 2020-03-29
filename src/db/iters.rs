use super::logs::Log;
use super::messagepack;
use rlua::Lua;

#[derive(Debug, PartialEq, Eq)]
pub struct Itr {
    pub log: String,
    pub name: String,
    pub func: String,
    pub kind: ItrKind,
}

impl Itr {
    pub fn next(
        &self,
        log: &Log,
        offset: usize,
        count: usize,
    ) -> Result<Vec<Vec<u8>>, messagepack::Error> {
        let lua = Lua::new();
        lua.context(|lua_ctx| {
            // TODO: this will panic if count is out of bounds.
            // Implement `get` on Log and return None if nothing exists.
            let results = (0..count)
                .map(|i| {
                    let msg = log[offset + i].clone();
                    // Deserializes message into the "msg" global var in the Lua vm.
                    let result = messagepack::unpack(lua_ctx, msg);

                    vec![0x00] // Placeholder until serialization code
                })
                .collect::<Vec<Vec<u8>>>();

            Ok(results)
        })
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
