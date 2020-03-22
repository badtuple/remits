use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::SystemTime;

use super::errors::Error;

/// The Manifest is a file at the root of the database directory that is used
/// as a registry for database constructs such as Logs and Iters. It will map
/// the identifiers of those constructs to their corresponding files, along
/// with any metadata needed.
///
/// Right now the Manifest is held in memory, just like the rest of POC database
/// until we are happy with the interface.
#[derive(Debug, PartialEq, Eq)]
pub struct Manifest {
    /// List of all existing logs
    pub logs: HashMap<String, LogRegistrant>,

    /// List of all existing Iters
    /// TODO: Once Iters are built out, store the actual code so they can be
    /// rebuilt.  For now, it's just the identifier.
    pub itrs: HashMap<String, ItrRegistrant>,
}

impl Manifest {
    pub fn new() -> Self {
        Manifest {
            logs: HashMap::new(),
            itrs: HashMap::new(),
        }
    }

    pub fn add_log(&mut self, name: String) {
        self.logs
            .entry(name.clone())
            .or_insert_with(|| LogRegistrant {
                name,
                created_at: SystemTime::now(),
            });
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
            self.del_itr(name.clone(), itr.to_owned())
                .expect("Could not delete itrs associated with log");
        }
    }
    pub fn add_itr(
        &mut self,
        log: String,
        name: String,
        kind: String,
        func: String,
    ) -> Result<(), Error> {
        let entry = self.itrs.entry(name.clone());
        match entry {
            Entry::Occupied(e) => {
                let itr = e.get();
                if itr.log != log || itr.kind != kind || itr.func != func {
                    return Err(Error::ItrExistsWithSameName);
                }
            }
            Entry::Vacant(e) => {
                e.insert(ItrRegistrant {
                    log,
                    name,
                    kind,
                    func,
                });
            }
        };

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

        Ok(())
    }
}

/// The Manifest entry for a Log
#[derive(Debug, PartialEq, Eq)]
pub struct LogRegistrant {
    pub name: String,
    pub created_at: SystemTime,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ItrRegistrant {
    pub log: String,
    pub name: String,
    pub kind: String,
    pub func: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_manifest_new() {
        let manifest = Manifest::new();
        assert_eq!(
            manifest,
            Manifest {
                logs: HashMap::new(),
                itrs: HashMap::new(),
            }
        );
    }
    #[test]
    fn test_manifest_add_log() {
        let mut manifest = Manifest::new();
        manifest.add_log("test".to_owned());
        manifest.add_log("test2".to_owned());
        manifest.add_log("test3".to_owned());
        assert!(manifest.logs.contains_key("test"));
        assert!(manifest.logs.contains_key("test2"));
        assert!(manifest.logs.contains_key("test3"));
        assert_eq!(manifest.logs.contains_key("test1"), false);

        // This second add_log is here to make sure code does not panic
        manifest.add_log("test".to_owned());
    }
    #[test]
    fn test_manifest_add_itr() {
        let mut manifest = Manifest::new();
        let _ = manifest.add_itr(
            "test".to_owned(),
            "fun".to_owned(),
            "lua".to_owned(),
            "func".to_owned(),
        );
        let _ = manifest.add_itr(
            "test".to_owned(),
            "fun2".to_owned(),
            "lua".to_owned(),
            "func".to_owned(),
        );
        let _ = manifest.add_itr(
            "test".to_owned(),
            "fun3".to_owned(),
            "lua".to_owned(),
            "func".to_owned(),
        );
        assert!(manifest.itrs.contains_key("fun"));
        assert!(manifest.itrs.contains_key("fun2"));
        assert!(manifest.itrs.contains_key("fun3"));
        assert_eq!(manifest.logs.contains_key("fun1"), false);

        let duplicate_error = manifest.add_itr(
            "test".to_owned(),
            "fun".to_owned(),
            "lua".to_owned(),
            "func2".to_owned(),
        );
        assert_eq!(
            format!("{:?}", duplicate_error),
            format!("Err(ItrExistsWithSameName)")
        );
    }

    #[test]
    fn test_manifest_del_itr() {
        let mut manifest = Manifest::new();
        // Normal
        let _ = manifest.add_itr(
            "test".to_owned(),
            "fun".to_owned(),
            "lua".to_owned(),
            "func".to_owned(),
        );
        assert!(manifest.itrs.contains_key("fun"));
        let _ = manifest.del_itr("test".to_owned(), "fun".to_owned());
        assert_eq!(manifest.logs.contains_key("fun"), false);

        // Function doesnt exist log does
        let does_not_exist_error = manifest.del_itr("test".to_owned(), "fun".to_owned());
        assert_eq!(
            format!("{:?}", does_not_exist_error),
            format!("Err(ItrDoesNotExist)")
        );
        // Neither function or log exist
        let _ = manifest.add_itr(
            "test".to_owned(),
            "fun".to_owned(),
            "lua".to_owned(),
            "func".to_owned(),
        );

        let log_does_not_exist_error = manifest.del_itr("test1".to_owned(), "fun".to_owned());
        assert_eq!(
            format!("{:?}", log_does_not_exist_error),
            format!("Err(ItrDoesNotExist)")
        );
    }
}
