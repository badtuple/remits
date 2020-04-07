use argh::FromArgs;
use env_logger::{Builder, Target};
use serde::{Deserialize, Serialize};

/// Server options
#[derive(Clone, Debug, Serialize, Deserialize, FromArgs)]
pub struct RemitsConfig {
    #[argh(option, short = 'p')]
    /// what port to start remits on
    port: Option<String>,
    // v can change dont care
    #[argh(option, short = 'v')]
    /// verbosity of logs
    log_level: Option<String>,
}

impl RemitsConfig {
    fn new() -> Self {
        confy::load("remits").expect("could not load config")
    }

    fn update_from_flags(&mut self) -> Self {
        let flags: RemitsConfig = argh::from_env();

        // This one must be first so debug logs work the rest of the way down
        setup_logger(self.log_level.clone(), flags.log_level);

        if flags.port.is_some() {
            debug!(
                "Replacing config option \"port\":{} with flag \"-p/--port\":{}",
                self.port.as_ref().unwrap(),
                flags.port.as_ref().unwrap()
            );
            self.port = flags.port;
        }

        self.clone()
    }

    pub fn addr(&self) -> String {
        format!("0.0.0.0:{}", self.clone().port.expect("no port defined"))
    }
}

impl ::std::default::Default for RemitsConfig {
    fn default() -> Self {
        Self {
            port: Some("4242".into()),
            log_level: Some("info".into()),
        }
    }
}

fn setup_logger(config_level: Option<String>, flag_level: Option<String>) {
    let log_level = &flag_level.unwrap_or_else(|| {
        config_level
            .as_ref()
            .unwrap_or(&"info".to_owned())
            .to_string()
    });
    Builder::new()
        .parse_filters(log_level)
        .target(Target::Stdout)
        .format_timestamp_nanos()
        .init();

    info!("log level set to {}", log_level);
}

pub fn load() -> RemitsConfig {
    RemitsConfig::new().update_from_flags()
}
