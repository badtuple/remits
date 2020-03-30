use super::logs::Log;
use super::Error;
use std::io::Cursor;

#[derive(Debug, PartialEq, Eq)]
pub struct Itr {
    pub log: String,
    pub name: String,
    pub func: String,
    pub kind: ItrKind,
}

impl Itr {
    pub fn next(&self, log: &Log, offset: usize, count: usize) -> Result<Vec<Vec<u8>>, Error> {
        let mut output: Vec<Vec<u8>> = Vec::with_capacity(count);
        let mut error: Option<Error> = None;

        let lua = rlua::Lua::new();
        lua.context(|ctx| {
            let globals = ctx.globals();
            for i in 0..count {
                // TODO: this will panic if count is out of bounds.
                // Implement `get` on Log and return None if nothing exists.
                let msg = log[offset + i].clone();
                trace!("pulled msg from log: {:?}", msg);

                let mut deserializer = rmp_serde::decode::Deserializer::new(Cursor::new(msg.clone()));
                let serializer = rlua_serde::ser::Serializer { lua: ctx };
                let lua_msg = match serde_transcode::transcode(&mut deserializer, serializer) {
                    Ok(msg) => msg,
                    Err(e) => {
                        debug!("error transcoding msgpack to lua: {:?}", e);
                        error = Some(Error::InvalidMsgPack);
                        break;
                    }
                };

                globals.set("msg", lua_msg);
                let res = ctx.load(&*self.func).eval::<rlua::Value>();
                if let Err(e) = res {
                    debug!("error running lua: {:?} {:?}", e, msg);
                    error = Some(Error::ErrRunningLua);
                    break;
                };

                let mut buf: Vec<u8> = vec![];
                let value = res.expect("couldnt unwrap response from eval");
                let deserializer = rlua_serde::de::Deserializer { value: value.clone() };
                let mut serializer = rmp_serde::encode::Serializer::new(&mut buf);
                match serde_transcode::transcode(deserializer, &mut serializer) {
                    Ok(ok) => info!("printing ok: {:?}", ok),
                    Err(e) => {
                        debug!("error transcoding lua to msgpack: {:?} {:?}", e, value);
                        error = Some(Error::ErrReadingLuaResponse);
                    },
                };

                output.push(buf);
            }
        });

        if let Some(e) = error {
            return Err(e);
        }

        Ok(output)
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
