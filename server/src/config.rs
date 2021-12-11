use std::default::Default;

use serde::{Deserialize, Serialize};

/// Server options
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// what port to start remits on
    pub port: String,

    /// Verbosity of server logging. Options: [ "info", "debug", "trace" ]
    pub log_level: String,

    /// directory that contains the db
    pub db_path: String,
}

impl Config {
    pub fn load(_path: &str) -> Result<Self, Error> {
        // TODO: load json config from path.
        // TODO: Verify well-formedness and output useful errors.
        Ok(Config::default())
    }

    pub fn addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: "4242".into(),
            log_level: "info".into(),
            db_path: "/tmp/remits".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    //NoConfigFoundAtPath,
//MalformedConfig,
}
