use uuid::Uuid;

pub enum ActionEvent {
    ConnectionEvent(ConnectionEvent),
    MovementEvent(MovementEvent),
}

pub struct Update {
    pub uuid: Uuid,
    pub event: UpdateEvent,
}

pub enum UpdateEvent {
    MovementEvent(MovementEvent),
}

pub enum ConnectionEvent {
    ClientConnected(Uuid),
    ClientDisconnected(Uuid),
}

#[derive(Clone, Debug)]
pub struct MovementEvent {
    pub uuid: Uuid,
    pub position: nalgebra::Point3<f64>,
}