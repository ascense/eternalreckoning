use serde::{Serialize, Deserialize};

use eternalreckoning_core::util::logging::LoggingConfig;

use crate::server::ServerConfig;

#[derive(Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub logging: LoggingConfig,
    pub server: ServerConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            logging: LoggingConfig::default(),
            server: ServerConfig::default(),
        }
    }
}