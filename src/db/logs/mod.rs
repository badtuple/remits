use crate::errors::Error;
use segment::Segment;
use serde_cbor::{Error as CborError, Value as CborValue};
use std::ops::Index;
use std::path::PathBuf;

mod segment;

#[derive(Debug)]
pub struct Log {
    path: PathBuf,

    /// The Segment that is currently being written to.
    active_segment: Segment,
    data: Vec<Vec<u8>>,
}

impl Log {
    pub fn new(mut path: PathBuf, name: &str) -> Self {
        path.push("logs");
        path.push(name);

        std::fs::create_dir_all(&path).expect("could not create log directory");

        let active_segment = Segment::get_active_for(path.clone());
        Log {
            path,
            active_segment,
            data: vec![],
        }
    }

    pub fn add_msg(&mut self, msg: Vec<u8>) -> Result<(), Error> {
        let res: Result<CborValue, CborError> = serde_cbor::from_reader(&mut &*msg);
        if res.is_err() {
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
    use crate::test_util::temp_db_path;

    #[test]
    fn test_add_valid_cbor_msg() {
        let mut log = Log::new(temp_db_path().into(), "test_log");
        let msg = vec![0x19, 0x03, 0xE8];
        if let Err(e) = log.add_msg(msg) {
            panic!("threw error for valid cbor: {:?}", e);
        };
    }

    #[test]
    fn test_add_invalid_cbor_msg() {
        let mut log = Log::new(temp_db_path().into(), "test_log");
        let buf = vec![0x1a, 0x01, 0x02];
        assert_eq!(log.add_msg(buf).is_err(), true);
    }
}
