use std::sync::mpsc::Sender;
use specs::{
    DispatcherBuilder,
    World,
    WorldExt,
    world::Builder,
};
use futures::sync::mpsc::UnboundedSender;

use crate::input::MouseEuler;
use super::event::{
    Event,
    Update,
};
use super::component::{
    Health,
    Jump,
    Movement,
    Name,
    Position,
    ServerID,
    Velocity,
};
use super::resource::{
    InputMap,
    TickLength,
};
use super::system::{
    UpdateInputs,
    Physics,
    PlayerMovement,
    UpdateSender,
    UpdateWorld,
};

use eternalreckoning_core::simulation::Simulation;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SimulationConfig {
    pub gravity: f64,
    pub movement_speed: f64,
    pub jump_force: f64,
}

impl Default for SimulationConfig {
    fn default() -> SimulationConfig {
        SimulationConfig {
            gravity: 0.48,
            movement_speed: 6.0,
            jump_force: 10.35,
        }
    }
}

pub fn build_simulation<'a, 'b>(
    mut config: SimulationConfig,
    update_tx: Sender<Update>,
    net_update_tx: UnboundedSender<Update>,
    tick_length: std::time::Duration,
) -> Simulation<'a, 'b, Event>
{
    let mut world = World::new();

    let tick_length = TickLength(tick_length);

    config.gravity = tick_length.scale_to(config.gravity);
    config.movement_speed = tick_length.scale_to(config.movement_speed);
    config.jump_force = tick_length.scale_to(config.jump_force);

    world.insert(InputMap::default());
    world.insert(MouseEuler::default());
    world.insert(tick_length);

    world.register::<Health>();
    world.register::<Jump>();
    world.register::<Movement>();
    world.register::<Name>();
    world.register::<Position>();
    world.register::<ServerID>();
    world.register::<Velocity>();

    world.create_entity()
        .with(Name("Player".to_string()))
        .with(Health(100))
        .with(Position(nalgebra::Point3::new(0.0, 0.0, 0.0)))
        .with(Velocity(nalgebra::Vector3::new(0.0, 0.0, 0.0)))
        .with(Movement { speed: config.movement_speed })
        .with(Jump { force: config.jump_force })
        .build();

    let dispatcher = DispatcherBuilder::new()
        .with(UpdateInputs, "update_inputs", &[])
        .with(PlayerMovement, "player_movement", &["update_inputs"])
        .with(Physics::new(config.gravity), "physics", &["player_movement"])
        .with(UpdateSender::new(update_tx, net_update_tx), "update_sender", &["player_movement", "physics"])
        .with(UpdateWorld, "update_world", &[])
        .build();

    Simulation::new(dispatcher, world)
}