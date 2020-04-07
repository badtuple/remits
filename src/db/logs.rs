use crate::errors::Error;
use serde_cbor::{Error as CborError, Value as CborValue};
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
        let res: Result<CborValue, CborError> = serde_cbor::from_reader(&mut &*msg);
        if let Err(_e) = res {
            return Err(Error::MsgNotValidCbor);
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
    use super::*;

    #[test]
    fn test_add_valid_cbor_msg() {
        let mut log = Log::new();
        let msg = vec![0x19, 0x03, 0xE8];
        if let Err(e) = log.add_msg(msg) {
            panic!("threw error for valid messagepack: {:?}", e);
        };
    }

    #[test]
    fn test_add_invalid_cbor_msg() {
        let mut log = Log::new();
        let buf = vec![0x1a, 0x01, 0x02];
        assert_eq!(log.add_msg(buf).is_err(),true);
    }
}
