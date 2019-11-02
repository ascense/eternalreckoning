use specs::prelude::*;

#[derive(Copy, Clone)]
pub enum ClientState {
    Connecting,
    Connected,
}

pub struct Client {
    pub state: ClientState,
}

impl Component for Client {
    type Storage = VecStorage<Self>;
}

impl Client {
    pub fn new() -> Client {
        Client { state: ClientState::Connecting }
    }
}