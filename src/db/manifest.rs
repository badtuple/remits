use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;

use super::iters::Itr;
use crate::commands::IteratorKind;
use crate::errors::Error;

/// The Manifest is a file at the root of the database directory that is used
/// as a registry for database constructs such as Logs and Iters. It will map
/// the identifiers of those constructs to their corresponding files, along
/// with any metadata needed.
///
/// Right now the Manifest is held in memory, just like the rest of POC database
/// until we are happy with the interface.
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub logs: HashMap<String, LogRegistrant>,
    pub itrs: HashMap<String, Itr>,

    #[serde(skip)]
    file_handle: Option<File>,
}

impl Manifest {
    pub fn new(path: &Path) -> Self {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(false)
            .open(path)
            .expect("could not create manifest file");

        let mut manifest = Manifest {
            logs: HashMap::new(),
            itrs: HashMap::new(),
            file_handle: Some(file),
        };

        if let Err(e) = manifest.flush_to_file() {
            error!("could not flush manifest to disk: {}", e);
            panic!("shutting down");
        };

        manifest
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .append(false)
            .open(path)?;
        let mut m: Manifest = serde_cbor::from_reader(&file)?;
        m.file_handle = Some(file);
        Ok(m)
    }

    fn flush_to_file(&mut self) -> Result<(), std::io::Error> {
        let bytes = serde_cbor::to_vec(&self).expect("could not serialize manifest");
        let mut handle = self.file_handle.as_ref().unwrap();

        handle.seek(std::io::SeekFrom::Start(0))?;
        handle.write_all(&*bytes)?;
        handle.set_len(bytes.len() as u64)?;
        handle.sync_data()?;
        Ok(())
    }

    pub fn add_log(&mut self, name: String) {
        self.logs
            .entry(name.clone())
            .or_insert_with(|| LogRegistrant {
                name,
                created_at: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("could not get system time")
                    .as_secs() as usize,
            });

        self.flush_to_file().expect("could not flush manifest");
    }

    pub fn del_log(&mut self, name: String) {
        self.logs.remove(&name.clone());
        let to_be_deleted: Vec<String> = self
            .itrs
            .iter()
            .filter(|(_, itr)| itr.log == name)
            .map(|(_, x)| x.name.clone())
            .collect();

        for itr in to_be_deleted.iter() {
            self.del_itr(name.clone(), itr.into())
                .expect("Could not delete itrs associated with log");
        }

        self.flush_to_file().expect("could not flush manifest");
    }

    pub fn add_itr(
        &mut self,
        log: String,
        name: String,
        kind: IteratorKind,
        func: String,
    ) -> Result<(), Error> {
        let itr = Itr {
            log,
            name: name.clone(),
            kind,
            func,
        };

        let entry = self.itrs.entry(name);
        match entry {
            Entry::Occupied(e) => {
                let stored_itr = e.get();
                if *stored_itr != itr {
                    return Err(Error::ItrExistsWithSameName);
                };
            }
            Entry::Vacant(e) => {
                e.insert(itr);
            }
        };

        self.flush_to_file().expect("could not flush manifest");
        Ok(())
    }

    pub fn del_itr(&mut self, log: String, name: String) -> Result<(), Error> {
        let entry = self.itrs.entry(name);
        match entry {
            Entry::Occupied(e) => {
                let itr = e.get();
                if itr.log != log {
                    return Err(Error::ItrDoesNotExist);
                }
                let _ = e.remove();
            }
            Entry::Vacant(_e) => {
                return Err(Error::ItrDoesNotExist);
            }
        };

        self.flush_to_file().expect("could not flush manifest");
        Ok(())
    }
}

/// The Manifest entry for a Log
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogRegistrant {
    pub name: String,
    pub created_at: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    static path: &str = "/tmp/test_remits/manifest";

    #[test]
    fn test_manifest_add_log() {
        let mut manifest = Manifest::new(Path::new(path));
        manifest.add_log("test".into());
        manifest.add_log("test2".into());
        manifest.add_log("test3".into());
        assert!(manifest.logs.contains_key("test"));
        assert!(manifest.logs.contains_key("test2"));
        assert!(manifest.logs.contains_key("test3"));
        assert_eq!(manifest.logs.contains_key("test1"), false);

        // This second add_log is here to make sure code does not panic
        manifest.add_log("test".into());
    }

    #[test]
    fn test_manifest_add_itr() {
        let mut manifest = Manifest::new(Path::new(path));
        let _ = manifest.add_itr("test".into(), "fun".into(), "map".into(), "func".into());
        let _ = manifest.add_itr("test".into(), "fun2".into(), "map".into(), "func".into());
        let _ = manifest.add_itr("test".into(), "fun3".into(), "map".into(), "func".into());
        assert!(manifest.itrs.contains_key("fun"));
        assert!(manifest.itrs.contains_key("fun2"));
        assert!(manifest.itrs.contains_key("fun3"));
        assert_eq!(manifest.logs.contains_key("fun1"), false);

        let duplicate_error =
            manifest.add_itr("test".into(), "fun".into(), "map".into(), "func2".into());
        assert_eq!(
            format!("{:?}", duplicate_error),
            "Err(ItrExistsWithSameName)".to_string()
        );
    }

    #[test]
    fn test_manifest_del_itr() {
        let mut manifest = Manifest::new(Path::new(path));
        // Normal
        let _ = manifest.add_itr("test".into(), "fun".into(), "map".into(), "func".into());
        assert!(manifest.itrs.contains_key("fun"));
        let _ = manifest.del_itr("test".into(), "fun".into());
        assert_eq!(manifest.logs.contains_key("fun"), false);

        // Function doesnt exist log does
        let does_not_exist_error = manifest.del_itr("test".into(), "fun".into());
        assert_eq!(
            format!("{:?}", does_not_exist_error),
            "Err(ItrDoesNotExist)".to_string()
        );
        // Neither function or log exist
        let _ = manifest.add_itr("test".into(), "fun".into(), "map".into(), "func".into());

        let log_does_not_exist_error = manifest.del_itr("test1".into(), "fun".into());
        assert_eq!(
            format!("{:?}", log_does_not_exist_error),
            "Err(ItrDoesNotExist)".to_string()
        );
    }
}
