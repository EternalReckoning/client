use serde::Deserialize;

use eternalreckoning_core::util::logging::LoggingConfig;

use crate::client::ClientConfig;
use crate::input::MouseConfig;

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