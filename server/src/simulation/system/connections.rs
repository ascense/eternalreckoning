use specs::prelude::*;

use eternalreckoning_core::net::operation::{
    Operation,
};

use super::super::{
    component::{
        Client,
        Id,
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
        WriteStorage<'a, Id>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, events, mut clients, mut ids, mut positions) = data;

        for event in &*events {
            match event.op {
                Operation::ClConnectMessage(_) => {
                    log::info!("Client connected: {}", event.uuid);

                    let client = entities.create();

                    ids.insert(client, Id(event.uuid.clone()))
                        .unwrap_or_else(|err| {
                            log::error!(
                                "Failed to add id for client {}: {}",
                                event.uuid,
                                err
                            );
                            None
                        });

                    clients.insert(client, Client::new())
                        .unwrap_or_else(|err| {
                            log::error!(
                                "Failed to add state for client {}: {}",
                                event.uuid,
                                err
                            );
                            None
                        });

                    positions.insert(client, Position(
                        nalgebra::Point3::<f64>::new(0.0, 0.0, 0.0)
                    ))
                        .unwrap_or_else(|err| {
                            log::error!(
                                "Failed to add position for client {}: {}",
                                event.uuid,
                                err
                            );
                            None
                        });
                },
                _ => (),
            }
        }

        /*
        TODO
        ConnectionEvent::ClientDisconnected(uuid) => {
            log::info!("Client disconnected: {:?}", uuid);

            for (entity, client) in (&entities, &clients).join() {
                if &client.0 == uuid {
                    entities.delete(entity);
                    break;
                }
            }
        },
        */
    }
}
