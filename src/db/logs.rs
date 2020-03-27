use super::errors::Error;
use rmpv::decode::value::read_value;
use std::ops::Index;

#[derive(Debug, PartialEq, Eq)]
pub struct Log {
    data: Vec<Vec<u8>>,
}

impl Log {
    pub fn new() -> Self {
        Log { data: vec![] }
    }

    pub fn add_msg(&mut self, msg: Vec<u8>) -> Result<(), Error> {
        if read_value(&mut &*msg).is_err() {
            return Err(Error::MsgNotValidMessagePack);
        }
        self.data.push(msg);
        Ok(())
    }
}

impl Index<usize> for Log {
    type Output = Vec<u8>;

    fn index(&self, i: usize) -> &Vec<u8> {
        &self.data[i]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_add_valid_messagepack_msg() {
        let log = Log::new();
        let msg = vec![0x93, 0x00, 0x2a, 0xf7];
        if let Err(e) = log.add_msg(msg) {
            panic!("threw error for valid messagepack: {:?}", e);
        };
    }

    #[test]
    fn test_add_invalid_messagepack_msg() {
        let log = Log::new();
        // MessagePack encoded string,
        // but bytes in string aren't valid utf8
        let buf = vec![0xd9, 0x02, 0xc3, 0x28];
        if let Ok(_) = log.add_msg(buf) {
            panic!("invalid messagepack was allowed into log");
        };
    }
}
