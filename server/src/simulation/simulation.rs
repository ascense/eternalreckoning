use std::time::Duration;

use futures::sync::mpsc::UnboundedSender;
use specs::{
    DispatcherBuilder,
    World,
    WorldExt,
};
use uuid::Uuid;

use eternalreckoning_core::net::operation::Operation;

use super::Event;
use super::component::{
    Client,
    Health,
    Name,
    Position,
};
use super::system::{
    Connections,
    PlayerMovement,
    UpdateSender,
};

use eternalreckoning_core::simulation::Simulation;

pub fn build_simulation<'a, 'b>(
    net_tx: UnboundedSender<(Uuid, Operation)>,
    client_ttl_ms: u64,
) -> Simulation<'a, 'b, Event>
{
    let mut world = World::new();

    world.register::<Client>();
    world.register::<Health>();
    world.register::<Name>();
    world.register::<Position>();
    
    let dispatcher = DispatcherBuilder::new()
        .with(Connections::new(Duration::from_millis(client_ttl_ms)), "connections", &[])
        .with(PlayerMovement, "player_movement", &[])
        .with(UpdateSender::new(net_tx), "update_sender", &["player_movement"])
        .build();

    Simulation::new(dispatcher, world)
}