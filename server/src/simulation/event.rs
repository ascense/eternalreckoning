use uuid::Uuid;

use eternalreckoning_core::net::operation::Operation;

pub struct Event {
    pub uuid: Uuid,
    pub op: Operation,
}