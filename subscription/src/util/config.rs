use serde::{Serialize, Deserialize};

use eternalreckoning_core::util::logging::LoggingConfig;

#[derive(Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub logging: LoggingConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            logging: LoggingConfig::default(),
        }
    }
}