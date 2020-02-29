use std::time::Duration;
use std::thread;
use std::sync::mpsc::{
    channel,
    TryRecvError,
};

use failure::Error;
use futures::channel::mpsc::unbounded;

use crate::{
    eventloop,
    iohandler,
    networking,
    display::{
        Renderer,
        window::Window,
    },
    simulation::build_simulation,
    util::config,
};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ClientConfig {
    pub server_address: String,
    pub tick_rate: u64,
}

impl Default for ClientConfig {
    fn default() -> ClientConfig {
        ClientConfig {
            server_address: "127.0.0.1:6142".to_string(),
            tick_rate: 60,
        }
    }
}

pub fn main(config: config::Config) -> Result<(), Error> {
    let (event_tx, event_rx) = channel();
    let (net_update_tx, net_update_rx) = unbounded();
    let (main_update_tx, main_update_rx) = channel();

    log::info!("Creating window...");

    let window = Window::new(&config.display)?;

    log::info!("Initializing networking");
    
    let net_event_tx = event_tx.clone();
    let addr = config.client.server_address.clone();
    thread::spawn(move || {
        networking::connect(
            &addr,
            net_update_rx,
            net_event_tx
        );
        log::info!("Networking closed");
    });

    log::info!("Initializing IO");

    let (iohandler, io_channel) = iohandler::IOHandler::new();
    thread::spawn(move || {
        iohandler.run();
        log::info!("IO closed");
    });
    
    log::info!("Initializing simulation");
    
    let tick_length = Duration::from_millis(
        1000 / config.client.tick_rate
    );

    let sim_config = config.simulation.clone();
    thread::spawn(move || {
        let mut game = build_simulation(
            sim_config,
            main_update_tx,
            net_update_tx,
            tick_length
        );
        game.run(
            move || {
                match event_rx.try_recv() {
                    Ok(event) => {
                        Ok(Some(event))
                    },
                    Err(TryRecvError::Empty) => Ok(None),
                    Err(TryRecvError::Disconnected) => Err(()),
                }
            },
            tick_length
        )
            .unwrap_or_else(|_| {
                log::error!("Network thread disconnected")
            });
        log::info!("Simulation closed");
    });

    log::info!("Initializing rendering pipeline...");

    let (window, event_loop) = window.split();
    let renderer = Renderer::new(window, &event_loop, &config.display)?;

    log::info!("Entering main loop");
    
    eventloop::run(renderer, event_loop, config, event_tx, main_update_rx, io_channel)
}