use specs::prelude::*;
use uuid::Uuid;

pub struct Id(pub Uuid);

impl Component for Id {
    type Storage = VecStorage<Self>;
}