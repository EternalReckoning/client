use failure::Error;

#[derive(serde::Deserialize)]
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

#[derive(serde::Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

pub fn configure(config: &LoggingConfig) -> Result<(), Error> {
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
        .level(level)
        .chain(std::io::stdout());
    
    if let Some(ref path) = config.file {
        logging = logging.chain(
            fern::log_file(std::path::Path::new(path))?
        );
    }

    logging.apply()?;

    Ok(())
}