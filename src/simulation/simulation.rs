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
    collider::{self, Collider},
    Health,
    Jump,
    Model,
    Movement,
    Name,
    Position,
    ServerID,
    Velocity,
};
use super::resource::{
    ActiveCamera,
    ActiveCharacter,
    InputMap,
    TickLength,
};
use super::system::{
    CollisionDetection,
    CollisionResolver,
    Physics,
    PlayerMovement,
    UpdateInputs,
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
    pub min_collision_depth: f64,
    pub max_ground_slope: f64,
}

impl Default for SimulationConfig {
    fn default() -> SimulationConfig {
        SimulationConfig {
            gravity: 0.48,
            movement_speed: 6.0,
            jump_force: 10.35,
            min_collision_depth: 0.001,
            max_ground_slope: 0.2,
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

    world.register::<Collider>();
    world.register::<Health>();
    world.register::<Jump>();
    world.register::<Model>();
    world.register::<Movement>();
    world.register::<Name>();
    world.register::<Position>();
    world.register::<ServerID>();
    world.register::<Velocity>();

    // Floor collision plane
    world.create_entity()
        .with(Position(nalgebra::Point3::new(0.0, 0.0, 0.0)))
        .with(Collider::new(collider::ColliderType::Plane(
            -nalgebra::Vector3::y_axis()
        )))
        .with(Model::new("assets/floor.erm"))
        .build();

    // Player
    let player = world.create_entity()
        .with(Name("Player".to_string()))
        .with(Health(100))
        .with(Position(nalgebra::Point3::new(0.0, -1.0, 0.0)))
        .with(Velocity(nalgebra::Vector3::new(0.0, 0.0, 0.0)))
        .with(Movement { speed: config.movement_speed, on_ground: true })
        .with(Jump { force: config.jump_force })
        .with(Collider::new(collider::ColliderType::Sphere(1.0)))
        .with(Model {
            path: "assets/marker.erm".to_string(),
            offset: Some(nalgebra::Vector3::new(0.0, 1.0, 0.0))
        })
        .build();

    world.insert(ActiveCamera(Some(player)));
    world.insert(ActiveCharacter(Some(player)));

    world.create_entity()
        .with(Position(nalgebra::Point3::new(-5.5, -1.0, -7.0)))
        .with(Model {
            path: "assets/pillar.erm".to_string(),
            offset: Some(nalgebra::Vector3::new(0.0, 1.0, 0.0))
        })
        .with(Collider::new(collider::ColliderType::Sphere(1.0)))
        .build();

    world.create_entity()
        .with(Position(nalgebra::Point3::new(5.5, -1.0, -7.0)))
        .with(Model {
            path: "assets/pillar.erm".to_string(),
            offset: Some(nalgebra::Vector3::new(0.0, 1.0, 0.0))
        })
        .with(Collider::new(collider::ColliderType::Sphere(1.0)))
        .build();

    world.create_entity()
        .with(Position(nalgebra::Point3::new(0.0, -0.5, -9.0)))
        .with(Model {
            path: "assets/elf-spear.erm".to_string(),
            offset: Some(nalgebra::Vector3::new(0.0, 0.5, 0.0))
        })
        .with(Collider::new(collider::ColliderType::Sphere(0.5)))
        .build();

    let dispatcher = DispatcherBuilder::new()
        .with(UpdateInputs, "update_inputs", &[])
        .with(PlayerMovement, "player_movement", &["update_inputs"])
        .with(Physics::new(config.gravity), "physics", &["player_movement"])
        .with(
            CollisionDetection::new(config.min_collision_depth),
            "collision_detection",
            &["physics"]
        )
        .with(
            CollisionResolver::new(config.max_ground_slope),
            "collision_resolver",
            &["collision_detection"]
        )
        .with(
            UpdateSender::new(update_tx, net_update_tx),
            "update_sender",
            &["player_movement", "physics", "collision_detection", "collision_resolver"]
        )
        .with(UpdateWorld, "update_world", &[])
        .build();

    Simulation::new(dispatcher, world)
}