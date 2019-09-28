use serde::Deserialize;

use crate::client::ClientConfig;
use crate::input::MouseConfig;
use super::logging::LoggingConfig;

#[derive(Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub client: ClientConfig,
    pub logging: LoggingConfig,
    pub mouse: MouseConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            client: ClientConfig::default(),
            logging: LoggingConfig::default(),
            mouse: MouseConfig::default(),
        }
    }
}