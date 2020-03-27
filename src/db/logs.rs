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
    fn test_add_valid_messagepack_msg() {}

    #[test]
    fn test_add_invalid_messagepack_msg() {}
}
