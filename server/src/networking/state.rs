use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

pub struct State {
    pub id_to_addr: HashMap<Uuid, SocketAddr>,
    pub addr_to_id: HashMap<SocketAddr, Uuid>,
}

pub type SharedState = Arc<Mutex<State>>;

impl State {
    pub fn new() -> State {
        State {
            id_to_addr: HashMap::new(),
            addr_to_id: HashMap::new(),
        }
    }
}