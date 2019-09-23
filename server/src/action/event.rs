use uuid::Uuid;

pub enum ActionEvent {
    ConnectionEvent(ConnectionEvent),
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