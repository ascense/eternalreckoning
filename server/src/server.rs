use std::time::Duration;
use std::thread;
use std::sync::mpsc::{
    channel,
};

use failure::Error;
use futures::sync::mpsc::unbounded;

use crate::game::build_simulation;
use crate::networking::Server;
use crate::util::config::Config;

#[derive(serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServerConfig {
    pub tick_rate: u64,
    pub bind_address: String,
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig {
            tick_rate: 60,
            bind_address: "127.0.0.1:6142".to_string(),
        }
    }
}

pub fn main(config: Config) -> Result<(), Error> {
    let (event_tx, event_rx) = channel();
    let (update_tx, update_rx) = unbounded();

    let addr = config.server.bind_address.clone();
    thread::spawn(move || {
        let server = Server::new();
        server.run(&addr, update_rx, event_tx);
    });

    let tick_length = Duration::from_millis(
        1000 / config.server.tick_rate
    );

    let mut game = build_simulation(update_tx);
    game.run(event_rx, tick_length);

    Ok(())
}