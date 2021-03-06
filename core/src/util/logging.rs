use failure::Error;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> LoggingConfig {
        LoggingConfig {
            level: LogLevel::Info,
            file: None,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

pub fn configure(config: &LoggingConfig, component: &'static str) -> Result<(), Error> {
    let level = match config.level {
        LogLevel::Error => log::LevelFilter::Error,
        LogLevel::Warn => log::LevelFilter::Warn,
        LogLevel::Info => log::LevelFilter::Info,
        LogLevel::Debug => log::LevelFilter::Debug,
        LogLevel::Trace => log::LevelFilter::Trace,
    };

    let mut logging = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .level_for("eternalreckoning_core", level)
        .level_for(component, level)
        .chain(std::io::stdout());
    
    if let Some(ref path) = config.file {
        logging = logging.chain(
            fern::log_file(std::path::Path::new(path))?
        );
    }

    logging.apply()?;

    Ok(())
}