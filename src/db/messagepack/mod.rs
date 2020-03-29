use rlua::prelude::*;
use std::convert::TryInto;

/// The set_in_lua! macro handles setting arbitrary decoded bytes + types into a Lua construct.
/// Right now this is used for setting global variables, making entries into tables, and adding
/// things to arrays. We rely on this really heavily because otherwise, we'd have massive match
/// statements everywhere to handle every possible combination of Key-Type and Value-Type possible
/// in the interpreter.
macro_rules! set_in_lua {
    ($lua:expr, $table:expr, $key:expr, $val_typ:expr, $val_buf:expr) => {
        match $val_typ {
            Uint8 | Int8 | Uint16 | Int16 | Uint32 | Int32 | Uint64 | Int64 | PositiveFixnum
            | NegativeFixnum => $table.set($key, get_i64(&$val_buf)),
            Float | Double => $table.set($key, get_f64(&$val_buf)),
            Nil => $table.set($key, LuaValue::Nil),
            True => $table.set($key, true),
            False => $table.set($key, false),
            Raw8 | Raw16 | Raw32 | FixRaw => $table.set($key, $val_buf.clone()),
            Array16 | Array32 | FixMapForArray => {
                $table.set($key, messagepack_array_to_lua_table($lua, &$val_buf)?)
            }
            Map16 | Map32 | FixMapForHash => {
                $table.set($key, messagepack_map_to_lua_table($lua, &$val_buf)?)
            }
        }
        .expect("could not set value in lua vm");
    };
}

//msgpack = cmsgpack.pack(lua_object1, lua_object2, ..., lua_objectN)
//lua_object1, lua_object2, ..., lua_objectN = cmsgpack.unpack(msgpack)

/// Converts MessagePack encoded Binary to Lua Types and sets them in the running Lua Context with
/// under the variable name `msg`
///
/// Because the data for deserialized types is managed by Lua itself, there isn't a good way to
/// separate the Lua instantiation and the parsing.
pub fn unpack<'lua>(lua: rlua::Context<'lua>, msg: Vec<u8>) -> Result<(), Error> {
    use MPType::*;

    // The global namespace within the lua vm
    let globals = lua.globals();

    // A byte buffer we'll use to collect all the bytes for a type before reifying it.
    let mut buf: Vec<u8> = Vec::new();

    let mut lexer = Lexer::new(msg);
    while !lexer.is_finished() {
        let typ = lexer.next_bytes_for_lua_type(&mut buf)?;
        set_in_lua!(lua, globals, "msg", typ, buf);
        buf.clear();
    }

    Ok(())
}

fn get_i64(bytes: &Vec<u8>) -> i64 {
    let mut mini_buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    for i in 0..bytes.len() {
        mini_buf[i] = bytes[i];
    }
    i64::from_be_bytes(mini_buf)
}

fn get_f64(bytes: &Vec<u8>) -> f64 {
    let mut mini_buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    for i in 0..bytes.len() {
        mini_buf[i] = bytes[i];
    }
    f64::from_be_bytes(mini_buf)
}

fn messagepack_array_to_lua_table<'lua>(
    lua: rlua::Context<'lua>,
    bytes: &Vec<u8>,
) -> Result<rlua::Table<'lua>, Error> {
    use MPType::*;

    let array = lua.create_table().expect("couldnt create table");
    let mut index = 0;

    let mut buf = vec![];
    let mut lexer = Lexer::new(bytes.clone());
    while !lexer.is_finished() {
        let typ = lexer.next_bytes_for_lua_type(&mut buf)?;
        set_in_lua!(lua, array, index, typ, buf);
        buf.clear();
        index += 1;
    }

    Ok(array)
}

fn messagepack_map_to_lua_table<'lua>(
    lua: rlua::Context<'lua>,
    bytes: &Vec<u8>,
) -> Result<rlua::Table<'lua>, Error> {
    use MPType::*;

    let table = lua.create_table().expect("couldnt create table");

    let mut key_buf = vec![];
    let mut val_buf = vec![];

    let mut lexer = Lexer::new(bytes.clone());
    while !lexer.is_finished() {
        let key_typ = lexer.next_bytes_for_lua_type(&mut key_buf)?;
        let val_typ = lexer.next_bytes_for_lua_type(&mut val_buf)?;

        match key_typ {
            Uint8 | Int8 | Uint16 | Int16 | Uint32 | Int32 | Uint64 | Int64 | PositiveFixnum
            | NegativeFixnum => {
                let key = get_i64(&key_buf);
                set_in_lua!(lua, table, key, val_typ, val_buf);
            }
            Float | Double => {
                let key = get_f64(&key_buf);
                set_in_lua!(lua, table, key, val_typ, val_buf);
            }
            Nil => {
                let key = LuaValue::Nil;
                set_in_lua!(lua, table, key, val_typ, val_buf);
            }
            True => {
                let key = true;
                set_in_lua!(lua, table, key, val_typ, val_buf);
            }
            False => {
                let key = false;
                set_in_lua!(lua, table, key, val_typ, val_buf);
            }
            Raw8 | Raw16 | Raw32 | FixRaw => {
                let key = key_buf.clone();
                set_in_lua!(lua, table, key, val_typ, val_buf);
            }
            Array16 | Array32 | Map16 | Map32 | FixMapForArray | FixMapForHash => {
                return Err(Error::CannotHaveMapOrArrayAsMapKey);
            }
        }
    }

    Ok(table)
}

/// Returns true if msg is valid MessagePack
pub fn valid(msg: &Vec<u8>) -> bool {
    let mut lexer = Lexer::new(msg.clone());
    let mut buf = vec![];
    while !lexer.is_finished() {
        if let Err(_) = lexer.next_bytes_for_lua_type(&mut buf) {
            return false;
        };
    }
    true
}

// Lexer allows us to easily collect certain amounts of bytes based on the type
// specified by the leading byte.
struct Lexer {
    idx: usize,
    msg: Vec<u8>,
}

impl Lexer {
    fn new(msg: Vec<u8>) -> Self {
        Lexer { idx: 0, msg }
    }

    fn is_finished(&self) -> bool {
        self.idx >= self.msg.len()
    }

    fn next_bytes_for_lua_type(&mut self, lua_byte_buf: &mut Vec<u8>) -> Result<MPType, Error> {
        use MPType::*;

        let type_byte = self.next();
        let typ = type_byte.try_into()?;

        let bytes_of_data = match typ {
            Uint8 => 2,
            Int8 => 2,
            Uint16 => 3,
            Int16 => 3,
            Uint32 => 5,
            Int32 => 5,
            Uint64 => 9,
            Int64 => 9,
            Nil => 0,
            True => 1,
            False => 1,
            Float => 5,
            Double => 9,
            Raw8 => 2,
            Raw16 => 3,
            Raw32 => 5,
            Array16 => 3,
            Array32 => 5,
            Map16 => 3,
            Map32 => 5,
            FixRaw => type_byte & 0x1F,
            FixMapForArray => type_byte & 0xF,
            FixMapForHash => type_byte & 0xF,
            PositiveFixnum | NegativeFixnum => {
                lua_byte_buf.push(type_byte);
                0
            }
        };

        // Collects known sizes/lengths. This includes fixed-size types and the headers of variably
        // sized types.
        for _ in 0..bytes_of_data {
            lua_byte_buf.push(self.next());
        }

        if typ.is_fixed_size() {
            return Ok(typ);
        }

        // Collect data for variably sized types.
        let mut var_len_buf = [0x00; 8];
        match typ {
            Raw8 => {
                var_len_buf[0] = lua_byte_buf[0];
            }
            Raw16 | Array16 | Map16 => {
                var_len_buf[0] = lua_byte_buf[0];
                var_len_buf[1] = lua_byte_buf[1];
            }
            Raw32 | Array32 | Map32 => {
                var_len_buf[0] = lua_byte_buf[0];
                var_len_buf[1] = lua_byte_buf[1];
                var_len_buf[2] = lua_byte_buf[2];
                var_len_buf[3] = lua_byte_buf[3];
            }
            _ => unreachable!(),
        };

        let var_len = usize::from_be_bytes(var_len_buf);
        let mut bytes = match typ {
            Raw8 | Raw16 | Raw32 => self.consume_raw_bytes(var_len),
            Array16 | Array32 => self.consume_array_bytes(var_len)?,
            Map16 | Map32 => self.consume_map_bytes(var_len)?,
            _ => unreachable!(),
        };

        lua_byte_buf.clear();
        lua_byte_buf.append(&mut bytes);

        Ok(typ)
    }

    fn next(&mut self) -> u8 {
        let b = self.msg[self.idx];
        self.idx += 1;
        b
    }

    fn consume_raw_bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next()).collect()
    }

    // Elements corresponds to the number of elements in the array, not bytes.
    fn consume_array_bytes(&mut self, elements: usize) -> Result<Vec<u8>, Error> {
        let mut out_buf = vec![];
        for _ in 0..elements {
            self.next_bytes_for_lua_type(&mut out_buf)?;
        }
        return Ok(out_buf);
    }

    // Elements corresponds to the number of key/value pairs in the array, not bytes.
    fn consume_map_bytes(&mut self, elements: usize) -> Result<Vec<u8>, Error> {
        let mut out_buf = vec![];
        // We multiply elements by two because each key/value pair is serialized separately.
        for _ in 0..(elements * 2) {
            self.next_bytes_for_lua_type(&mut out_buf)?;
        }
        return Ok(out_buf);
    }
}

// MessagePack Types, connected to their byte_type representation
enum MPType {
    Uint8 = 0xCC,
    Int8 = 0xD0,
    Uint16 = 0xCD,
    Int16 = 0xD1,
    Uint32 = 0xCE,
    Int32 = 0xD2,
    Uint64 = 0xCF,
    Int64 = 0xD3,
    Nil = 0xC0,
    True = 0xC3,
    False = 0xC2,
    Float = 0xCA,
    Double = 0xCB,
    Raw8 = 0xD9,
    Raw16 = 0xDA,
    Raw32 = 0xDB,
    Array16 = 0xDC,
    Array32 = 0xDD,
    Map16 = 0xDE,
    Map32 = 0xDF,

    // No explicit byte to match
    PositiveFixnum,
    NegativeFixnum,
    FixRaw,
    FixMapForArray,
    FixMapForHash,
}

impl MPType {
    fn is_fixed_size(&self) -> bool {
        use MPType::*;

        match self {
            Raw8 | Raw16 | Raw32 | Array16 | Array32 | Map16 | Map32 => false,
            _ => true,
        }
    }
}

impl TryInto<MPType> for u8 {
    type Error = Error;
    fn try_into(self) -> Result<MPType, Error> {
        use MPType::*;
        let typ = match self {
            0xCC => Uint8,
            0xD0 => Int8,
            0xCD => Uint16,
            0xD1 => Int16,
            0xCE => Uint32,
            0xD2 => Int32,
            0xCF => Uint64,
            0xD3 => Int64,
            0xC0 => Nil,
            0xC3 => True,
            0xC2 => False,
            0xCA => Float,
            0xCB => Double,
            0xD9 => Raw8,
            0xDA => Raw16,
            0xDB => Raw32,
            0xDC => Array16,
            0xDD => Array32,
            0xDE => Map16,
            0xDF => Map32,
            _ => {
                /* types that can't be identified by first byte alone */
                if (self & 0x80) == 0 {
                    PositiveFixnum
                } else if (self & 0xE0) == 0xE0 {
                    NegativeFixnum
                } else if (self & 0xE0) == 0xA0 {
                    FixRaw
                } else if (self & 0xF0) == 0x90 {
                    FixMapForArray
                } else if (self & 0xF0) == 0x80 {
                    FixMapForHash
                } else {
                    return Err(Error::UnrecognizedType);
                }
            }
        };
        Ok(typ)
    }
}

pub enum Error {
    UnrecognizedType,
    CannotHaveMapOrArrayAsMapKey,
}
