use specs::{
    DispatcherBuilder,
    World,
    WorldExt,
};

use crate::action::ActionEvent;
use super::component::Position;
use super::system::{
    Connections,
    PlayerMovement,
};

use eternalreckoning_core::simulation::Simulation;

pub fn build_simulation<'a, 'b>() -> Simulation<'a, 'b, ActionEvent> {
    let mut world = World::new();

    world.register::<Position>();
    
    let dispatcher = DispatcherBuilder::new()
        .with(Connections, "connections", &[])
        .with(PlayerMovement, "player_movement", &[])
        .build();

    Simulation::new(dispatcher, world)
}