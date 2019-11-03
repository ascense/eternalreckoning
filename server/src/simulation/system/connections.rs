use std::time::{
    Duration,
    Instant,
};

use specs::prelude::*;

use eternalreckoning_core::net::operation::{
    Operation,
};
use eternalreckoning_core::simulation::TickTime;

use super::super::{
    component::{
        Client,
        Id,
        Position,
    },
    EventQueue,
};

pub struct Connections {
    ttl: Duration,
}

impl Connections {
    pub fn new(ttl: Duration) -> Connections {
        Connections { ttl }
    }
}

impl<'a> System<'a> for Connections {
    type SystemData = (
        Entities<'a>,
        Read<'a, TickTime>,
        Read<'a, EventQueue>,
        WriteStorage<'a, Client>,
        WriteStorage<'a, Id>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            tick_time,
            events,
            mut clients,
            mut ids,
            mut positions,
        ) = data;

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

                    clients.insert(client, Client::new(Instant::now() + self.ttl))
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
                Operation::ClSync(_) => {
                    for (id, client) in (&ids, &mut clients).join() {
                        if id.0 == event.uuid {
                            client.lifetime = tick_time.0 + self.ttl;
                            break;
                        }
                    }
                },
                Operation::ClMoveSetPosition(_) => {
                    for (id, client) in (&ids, &mut clients).join() {
                        if id.0 == event.uuid {
                            client.lifetime = tick_time.0 + self.ttl;
                            break;
                        }
                    }
                },
                Operation::DisconnectMessage => {
                    for (id, client) in (&ids, &mut clients).join() {
                        if id.0 == event.uuid {
                            client.lifetime = tick_time.0;
                            break;
                        }
                    }
                },
                _ => (),
            }
        }

        for (entity, id, client) in (&entities, &ids, &clients).join() {
            if client.lifetime <= tick_time.0 {
                log::info!("Client disconnected: {}", id.0);
                entities.delete(entity)
                    .unwrap_or_else(|err| {
                        log::error!(
                            "Failed to drop disconnected client {}: {}",
                            id.0,
                            err
                        );
                    });
            }
        }
    }
}
