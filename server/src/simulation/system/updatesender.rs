use futures::sync::mpsc::UnboundedSender;
use specs::prelude::*;
use uuid::Uuid;

use eternalreckoning_core::net::operation::{
    self,
    Operation,
};

use super::super::component::{
    client::ClientState,
    Client,
    Id,
    Position,
    Health,
};

pub struct UpdateSender {
    sender: UnboundedSender<(Uuid, Operation)>,
}

impl UpdateSender {
    pub fn new(sender: UnboundedSender<(Uuid, Operation)>) -> UpdateSender {
        UpdateSender { sender }
    }

    fn send_connection_response<'a>(
        &self,
        _entities: &Entities<'a>,
        ids: &ReadStorage<'a, Id>,
        _pos: &ReadStorage<'a, Position>,
        _health: &ReadStorage<'a, Health>,
        clients: &mut WriteStorage<'a, Client>,
        entity: Entity
    ) {
        let uuid = match ids.get(entity) {
            Some(uuid) => &uuid.0,
            None => {
                log::error!("No UUID for client!");
                return;
            }
        };
        
        let op = Operation::SvConnectResponse(
            operation::SvConnectResponse { uuid: *uuid }
        );

        self.sender.unbounded_send((*uuid, op))
            .unwrap_or_else(|err| {
                log::error!("Failed to send update: {}", err);
            });
        
        if let Some(client) = clients.get_mut(entity) {
            client.state = ClientState::Connected;
        }
    }

    fn send_world_update<'a>(
        &self,
        entities: &Entities<'a>,
        ids: &ReadStorage<'a, Id>,
        pos: &ReadStorage<'a, Position>,
        health: &ReadStorage<'a, Health>,
        _clients: &mut WriteStorage<'a, Client>,
        entity: Entity
    ) {
        let uuid = match ids.get(entity) {
            Some(uuid) => &uuid.0,
            None => {
                log::error!("No UUID for client!");
                return;
            }
        };
        let mut updates = Vec::new();

        for (ent, id) in (entities, ids).join() {
            let mut data = Vec::new();

            if *uuid != id.0 {
                if let Some(pos) = pos.get(ent) {
                    data.push(operation::EntityComponent::Position(
                        pos.0.clone()
                    ));
                }
            }

            if let Some(health) = health.get(ent) {
                data.push(operation::EntityComponent::Health(
                    health.0
                ));
            }

            updates.push(operation::EntityUpdate {
                uuid: id.0,
                data,
            });
        }

        let op = Operation::SvUpdateWorld(
            operation::SvUpdateWorld { updates }
        );
        self.sender.unbounded_send((*uuid, op))
            .unwrap_or_else(|err| {
                log::error!("Failed to send update: {}", err);
            });
        
    }
}

impl<'a> System<'a> for UpdateSender {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Id>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Health>,
        WriteStorage<'a, Client>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            ids,
            pos,
            health,
            mut clients,
        ) = data;

        for ent in entities.join() {
            let state = {
                match clients.get(ent) {
                    Some(client) => client.state,
                    None => continue,
                }
            };

            match state {
                ClientState::Connecting => {
                    self.send_connection_response(
                        &entities,
                        &ids,
                        &pos,
                        &health,
                        &mut clients,
                        ent
                    );
                },
                ClientState::Connected => {
                    self.send_world_update(
                        &entities,
                        &ids,
                        &pos,
                        &health,
                        &mut clients,
                        ent
                    );
                },
            }
        }
    }
}