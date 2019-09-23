use std::sync::mpsc::Sender;
use specs::{
    DispatcherBuilder,
    World,
    WorldExt,
    world::Builder,
};

use super::event::{
    Event,
    Update,
};
use super::component::{
    Movement,
    Position,
};
use super::resource::InputMap;
use super::system::{
    UpdateInputs,
    PlayerMovement,
    UpdateSender,
};

use eternalreckoning_core::simulation::Simulation;

pub fn build_simulation<'a, 'b>(update_tx: Sender<Update>) -> Simulation<'a, 'b, Event> {
    let mut world = World::new();

    world.register::<Movement>();
    world.register::<Position>();

    world.insert(InputMap::default());

    world.create_entity()
        .with(Position(nalgebra::Point3::new(0.0, -1.0, 0.0)))
        .with(Movement { speed: 0.1 })
        .build();
    
    let dispatcher = DispatcherBuilder::new()
        .with(UpdateInputs, "update_inputs", &[])
        .with(PlayerMovement, "player_movement", &["update_inputs"])
        .with(UpdateSender::new(update_tx), "update_sender", &["player_movement"])
        .build();

    Simulation::new(dispatcher, world)
}