use serde::{Serialize, Deserialize};

use eternalreckoning_core::util::logging::LoggingConfig;

use crate::client::ClientConfig;
use crate::input::MouseConfig;
use crate::input::KeyMapConfig;
use crate::simulation::SimulationConfig;

#[derive(Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub client: ClientConfig,
    pub key_map: KeyMapConfig,
    pub logging: LoggingConfig,
    pub mouse: MouseConfig,
    pub simulation: SimulationConfig,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            client: ClientConfig::default(),
            key_map: KeyMapConfig::default(),
            logging: LoggingConfig::default(),
            mouse: MouseConfig::default(),
            simulation: SimulationConfig::default(),
        }
    }
}