use std::sync::mpsc::{
    channel,
    TryRecvError,
};
use std::time::Duration;
use std::thread;

use failure::{
    format_err,
    Error,
};
use futures::sync::mpsc::unbounded;

use crate::simulation::build_simulation;
use crate::simulation::Event;
use crate::networking::Server;
use crate::util::config::Config;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServerConfig {
    pub tick_rate: u64,
    pub bind_address: String,
    pub client_ttl_ms: u64,
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig {
            tick_rate: 60,
            bind_address: "127.0.0.1:6142".to_string(),
            client_ttl_ms: 500,
        }
    }
}

pub fn main(config: Config) -> Result<(), Error> {
    let (outbound_tx, outbound_rx) = unbounded();
    let (inbound_tx, inbound_rx) = channel();

    let addr = config.server.bind_address.clone();
    thread::spawn(move || {
        let server = Server::new();
        server.run(&addr, outbound_rx, inbound_tx);
    });

    let tick_length = Duration::from_millis(
        1000 / config.server.tick_rate
    );

    let mut game = build_simulation(
        outbound_tx,
        config.server.client_ttl_ms
    );

    game.run(
        move || {
            match inbound_rx.try_recv() {
                Ok((uuid, op)) => {
                    Ok(Some(Event { uuid, op }))
                },
                Err(TryRecvError::Empty) => Ok(None),
                Err(TryRecvError::Disconnected) => Err(()),
            }
        },
        tick_length
    )
        .map_err(|_| {
            format_err!("Network thread disconnected")
        })?;

    Ok(())
}