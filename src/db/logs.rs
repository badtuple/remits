use super::errors::Error;
use super::messagepack;
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
        if !messagepack::valid(&msg) {
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
    use super::*;

    #[test]
    fn test_add_valid_messagepack_msg() {
        let mut log = Log::new();
        let msg = vec![0x93, 0x00, 0x2a, 0xf7];
        if let Err(e) = log.add_msg(msg) {
            panic!("threw error for valid messagepack: {:?}", e);
        };
    }

    #[test]
    fn test_add_invalid_messagepack_msg() {
        let mut log = Log::new();
        let buf = vec![0x93, 0x00, 0x2a];
        if let Ok(_) = log.add_msg(buf) {
            panic!("invalid messagepack was allowed into log");
        };
    }
}
