use std::fmt::{
    Display,
    Formatter,
};

use uuid::Uuid;

#[derive(Clone)]
pub enum Operation {
    ClConnectMessage(ClConnectMessage),
    SvConnectResponse(SvConnectResponse),
    SvUpdateWorld(SvUpdateWorld),
    ClMoveSetPosition(ClMoveSetPosition),
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", match self {
            Operation::ClConnectMessage(_) => "(client) connect message",
            Operation::SvConnectResponse(_) => "(server) connect response",
            Operation::SvUpdateWorld(_) => "(server) world update",
            Operation::ClMoveSetPosition(_) => "(client) player movement",
        })
    }
}

#[derive(Clone)]
pub struct ClConnectMessage;

#[derive(Clone)]
pub struct SvConnectResponse {
    pub uuid: Uuid,
}

#[derive(Clone)]
pub struct ClMoveSetPosition {
    pub pos: nalgebra::Point3<f64>,
}

#[derive(Clone)]
pub struct SvUpdateWorld {
    pub updates: Vec<EntityUpdate>,
}

#[derive(Clone)]
pub struct EntityUpdate {
    pub uuid: Uuid,
    pub data: Vec<EntityComponent>,
}

#[derive(Clone)]
pub enum EntityComponent {
    Health(u64),
    Position(nalgebra::Point3<f64>),
}