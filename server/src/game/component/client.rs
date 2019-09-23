use specs::prelude::*;
use uuid::Uuid;

pub struct Client(pub Uuid);

impl Component for Client {
    type Storage = VecStorage<Self>;
}