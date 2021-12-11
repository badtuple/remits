use std::collections::{hash_map::Entry, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

/// A handle to the underlying storage layer.
// TODO: Right now this is in-memory only until we hammer out the API/semantics.
// There's no point writing super involved disk handling code before we know what we need.
// As such, there shouldn't be any public properties on Storage, only methods.
pub struct Storage {
    logs: HashMap<String, Vec<RawMessage>>,
}

impl Storage {
    /// Open Storage for usage. If no Storage exists at that location, it will be created.
    pub fn open(_path: &str) -> Result<Storage, Error> {
        Ok(Storage {
            logs: HashMap::new(),
        })
    }

    pub fn create_log(&mut self, name: String) -> Result<(), Error> {
        match self.logs.entry(name) {
            Entry::Occupied(_) => Err(Error::LogAlreadyExists),
            Entry::Vacant(e) => {
                e.insert(vec![]);
                Ok(())
            }
        }
    }

    pub fn add_message_to_log(&mut self, log_name: String, body: Vec<u8>) -> Result<(), Error> {
        match self.logs.entry(log_name) {
            Entry::Occupied(mut o) => {
                // TODO: We can't make a syscall on every message like this. It'll be dog slow.
                // Honestly it shouldn't even be in the storage layer, it's just convenient right
                // now.
                let ingest_time = ingest_time();
                o.get_mut().push(RawMessage { ingest_time, body });
                Ok(())
            }
            Entry::Vacant(_) => Err(Error::LogDoesNotExist),
        }
    }
}

fn ingest_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_millis() as u64
}

// Persisted variant of a Message.
// Right now this is only for the in-memory version.
pub struct RawMessage {
    // Ingest time in milliseconds
    pub ingest_time: u64,

    // Raw JSON bytes
    pub body: Vec<u8>,
}

#[derive(Clone, Debug)]
pub enum Error {
    LogAlreadyExists,
    LogDoesNotExist,
}
