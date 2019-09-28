use std::fs::File;
use std::io::Read;
use std::path::Path;

use toml;
use failure_derive::Fail;

#[derive(Debug, Fail)]
pub enum ConfigurationError {
    #[fail(display = "invalid arguments")]
    InvalidArguments,
    #[fail(display = "malformed configuration: {}", _0)]
    MalformedData(#[cause] toml::de::Error),
    #[fail(display = "unable to read configuration file: {}", _0)]
    IoError(#[cause] std::io::Error),
}

pub struct Config<T> {
    pub data: T,
}

impl<T> Config<T>
where
    T: serde::de::DeserializeOwned + std::default::Default,
{
    pub fn from_str(src: &str)
        -> Result<Config<T>, ConfigurationError>
    {
        let config: T = toml::from_str(src)
            .map_err(|e| ConfigurationError::MalformedData(e))?;

        Ok(Config { data: config })
    }

    pub fn from_file<P: AsRef<Path>>(path: P)
        -> Result<Config<T>, ConfigurationError>
    {
        let mut buffer = String::new();

        File::open(path)
            .map_err(|e| ConfigurationError::IoError(e))?
            .read_to_string(&mut buffer)
            .map_err(|e| ConfigurationError::IoError(e))?;

        Config::from_str(&buffer)
    }
}