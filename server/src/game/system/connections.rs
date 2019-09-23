use specs::prelude::*;

use crate::action::{ ActionEvent, ConnectionEvent };
use super::super::{
    component::{
        Client,
        Position,
    },
    EventQueue,
};

pub struct Connections;

impl<'a> System<'a> for Connections {
    type SystemData = (
        Entities<'a>,
        Read<'a, EventQueue>,
        WriteStorage<'a, Client>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, events, mut clients, mut positions) = data;

        for event in &*events {
            match event {
                ActionEvent::ConnectionEvent(ref event_data) => {
                    match event_data {
                        ConnectionEvent::ClientConnected(uuid) => {
                            println!("Client connected: {:?}", uuid);
                            
                            let client = entities.create();

                            clients.insert(client, Client(uuid.clone()));
                            positions.insert(client, Position(
                                nalgebra::Point3::<f64>::new(0.0, 0.0, 0.0)
                            ));
                        },
                        ConnectionEvent::ClientDisconnected(uuid) => {
                            println!("Client disconnected: {:?}", uuid);
                            /*
                            for client in (&client).join() {
                                if client.0 == uuid {
                                    entities.destroy(client);
                                }
                            }
                            */
                        },
                    };
                },
                _ => (),
            }
        }
    }
}