use std::fs::File;
use std::io::Read;

use toml;
use failure_derive::Fail;

#[derive(Debug, Fail)]
pub enum ConfigurationError {
    #[fail(display = "invalid arguments")]
    InvalidArguments,
    #[fail(display = "malformed configuration: {}", _0)]
    MalformedData(#[cause] toml::de::Error),
    #[fail(display = "unable to access configuration file: {}", path)]
    IoError {
        #[cause] cause: std::io::Error,
        path: String,
    },
}

pub struct Config<T> {
    pub data: T,
}

impl<T> Config<T>
where
    T: serde::ser::Serialize + serde::de::DeserializeOwned + std::default::Default,
{
    pub fn from_str(src: &str)
        -> Result<Config<T>, ConfigurationError>
    {
        let config: T = toml::from_str(src)
            .map_err(|e| ConfigurationError::MalformedData(e))?;

        Ok(Config { data: config })
    }

    pub fn from_file(path: &String)
        -> Result<Config<T>, ConfigurationError>
    {
        let mut buffer = String::new();

        File::open(path)
            .map_err(|e| ConfigurationError::IoError {
                cause: e,
                path: path.clone(),
            })?
            .read_to_string(&mut buffer)
            .map_err(|e| ConfigurationError::IoError {
                cause: e,
                path: path.clone(),
            })?;

        Config::from_str(&buffer)
    }

    pub fn write_default(path: &String)
        -> Result<Config<T>, ConfigurationError>
    {
        let config: T = T::default();

        let buffer = toml::to_string(&config).unwrap();
        std::fs::write(path, &buffer[..])
            .map_err(|e| ConfigurationError::IoError {
                cause:e ,
                path: path.clone(),
            })?;
        
        Ok(Config { data: config })
    }
}