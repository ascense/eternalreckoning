use std::fmt::{
    Display,
    Formatter,
};

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

pub struct ClConnectMessage;
pub struct SvConnectResponse;

pub struct ClMoveSetPosition {
    pub pos: nalgebra::Point3<f64>,
}

pub struct SvUpdateWorld {
    pub updates: Vec<EntityUpdate>,
}

pub enum EntityUpdate {
    EntityMoved(nalgebra::Point3<f64>),
}