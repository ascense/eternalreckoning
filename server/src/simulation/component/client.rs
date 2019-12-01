use std::time::Instant;

use specs::prelude::*;

#[derive(Copy, Clone)]
pub enum ClientState {
    Connecting,
    Connected,
}

pub struct Client {
    pub state: ClientState,
    pub lifetime: Instant,
}

impl Component for Client {
    type Storage = VecStorage<Self>;
}

impl Client {
    pub fn new(lifetime: Instant) -> Client {
        Client { state: ClientState::Connecting, lifetime }
    }
}