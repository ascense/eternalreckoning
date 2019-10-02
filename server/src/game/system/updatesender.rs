use futures::sync::mpsc::UnboundedSender;

use specs::prelude::*;

use crate::action::event::{self, Update};
use super::super::component::{
    Client,
    Position,
};

pub struct UpdateSender {
    sender: UnboundedSender<Update>,
}

impl UpdateSender {
    pub fn new(sender: UnboundedSender<Update>) -> UpdateSender {
        UpdateSender { sender }
    }
}

impl<'a> System<'a> for UpdateSender {
    type SystemData = (ReadStorage<'a, Client>, ReadStorage<'a, Position>);

    fn run(&mut self, data: Self::SystemData) {
        let (clients, pos) = data;

        for (client, pos) in (&clients, &pos).join() {
            for receiver in clients.join() {
                self.sender.send(
                    Update {
                        uuid: receiver.0,
                        event: event::UpdateEvent::MovementEvent(
                            event::MovementEvent {
                                uuid: client.0,
                                position: pos.0.clone(),
                            },
                        ),
                    }
                ).map_err(|err| {
                    log::error!("Failed to send update: {}", err);
                });
            }
        }
    }
}