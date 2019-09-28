use serde::Deserialize;

use crate::server::ServerConfig;
use super::logging::LoggingConfig;

#[derive(Deserialize)]
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