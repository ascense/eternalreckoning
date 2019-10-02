use futures::sync::mpsc::UnboundedSender;
use specs::{
    DispatcherBuilder,
    World,
    WorldExt,
};

use crate::action::{ActionEvent, Update};
use super::component::Position;
use super::system::{
    Connections,
    PlayerMovement,
    UpdateSender,
};

use eternalreckoning_core::simulation::Simulation;

pub fn build_simulation<'a, 'b>(update_tx: UnboundedSender<Update>) -> Simulation<'a, 'b, ActionEvent> {
    let mut world = World::new();

    world.register::<Position>();
    
    let dispatcher = DispatcherBuilder::new()
        .with(Connections, "connections", &[])
        .with(PlayerMovement, "player_movement", &[])
        .with(UpdateSender::new(update_tx), "update_sender", &["player_movement"])
        .build();

    Simulation::new(dispatcher, world)
}