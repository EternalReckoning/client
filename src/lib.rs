pub mod util;
pub mod input;
pub mod loaders;
pub mod networking;
pub mod renderer;
pub mod simulation;
pub mod window;

mod client;

use failure::Error;
use failure::format_err;

use eternalreckoning_core::util::config::Config;
use eternalreckoning_core::util::logging;

pub struct Bootstrap {
    pub args: Vec<String>,
    pub config: Option<String>,
}

pub fn run(bootstrap: Bootstrap) -> Result<(), Error> {
    let config = initialize(bootstrap)?;

    client::main(config)?;

    Ok(())
}

fn initialize(bootstrap: Bootstrap)
    -> Result<util::config::Config, Error>
{
    let config = get_configuration(bootstrap)?;
    let config = config.data;

    logging::configure(&config.logging, "eternalreckoning_client")?;

    Ok(config)
}

fn get_configuration(bootstrap: Bootstrap)
    -> Result<Config<util::config::Config>, Error>
{
    match bootstrap.config {
        Some(path) => Ok(Config::<util::config::Config>::from_file(&path)?),
        None => Err(format_err!("no configuration file path provided")),
    }
}