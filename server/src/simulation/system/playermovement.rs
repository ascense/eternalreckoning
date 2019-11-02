use specs::prelude::*;
use uuid::Uuid;

use eternalreckoning_core::net::operation::{
    self,
    Operation,
};

use super::super::{
    component::{
        Id,
        Position,
    },
    EventQueue,
};

pub struct PlayerMovement;

impl<'a> System<'a> for PlayerMovement {
    type SystemData = (
        Read<'a, EventQueue>,
        ReadStorage<'a, Id>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (events, ids, mut pos) = data;

        let mut movement = Vec::<(&Uuid, &operation::ClMoveSetPosition)>::new();

        for event in &*events {
            match event.op {
                Operation::ClMoveSetPosition(ref data) => {
                    movement.push((&event.uuid, data));
                },
                _ => (),
            }
        }

        for (id, pos) in (&ids, &mut pos).join() {
            for event in &movement {
                if id.0 == *event.0 {
                    pos.0.x = event.1.pos.x;
                    pos.0.y = event.1.pos.y;
                    pos.0.z = event.1.pos.z;
                }
            }
        }
    }
}