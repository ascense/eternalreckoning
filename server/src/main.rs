use std::time::Duration;
use std::thread;
use std::sync::mpsc::{
    channel,
};

use eternalreckoning_server::game::build_simulation;
use eternalreckoning_server::networking::Server;

const TICK_RATE: u64 = 60;

fn main() {
    let (event_tx, event_rx) = channel();

    thread::spawn(move || {
        let server = Server::new();

        server.run(event_tx);
    });

    let tick_length = Duration::from_millis(
        1000 / TICK_RATE
    );

    let mut game = build_simulation();
    game.run(event_rx, tick_length);
}