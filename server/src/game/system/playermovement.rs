use specs::prelude::*;

use crate::action::{ ActionEvent, MovementEvent };
use super::super::{
    component::{
        Client,
        Position,
    },
    EventQueue,
};

pub struct PlayerMovement;

impl<'a> System<'a> for PlayerMovement {
    type SystemData = (
        Read<'a, EventQueue>,
        ReadStorage<'a, Client>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (events, client, mut pos) = data;

        let mut movement = Vec::<MovementEvent>::new();

        for event in &*events {
            match event {
                ActionEvent::MovementEvent(ref event_data) => {
                    movement.push(event_data.clone());
                },
                _ => (),
            }
        }

        for (client, pos) in (&client, &mut pos).join() {
            for event in &movement {
                if client.0 == event.uuid {
                    pos.0.x = event.position.x;
                    pos.0.y = event.position.y;
                    pos.0.z = event.position.z;
                }
            }
        }
    }
}