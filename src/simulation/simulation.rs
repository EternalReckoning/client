use std::sync::mpsc::Sender;
use specs::{
    DispatcherBuilder,
    World,
    WorldExt,
    world::Builder,
};
use futures::sync::mpsc::UnboundedSender;

use crate::input::MouseEuler;
use crate::loaders::heightmap_from_bmp;
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
    Texture,
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
use super::PhysicsConfig;

use eternalreckoning_core::simulation::Simulation;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SimulationConfig {
    pub movement_speed: f64,
    pub jump_force: f64,
    pub physics: PhysicsConfig,
}

impl Default for SimulationConfig {
    fn default() -> SimulationConfig {
        SimulationConfig {
            movement_speed: 6.0,
            jump_force: 10.35,
            physics: PhysicsConfig::default(),
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

    config.movement_speed = tick_length.scale_to(config.movement_speed);
    config.jump_force = tick_length.scale_to(config.jump_force);
    
    config.physics.gravity = tick_length.scale_to(config.physics.gravity);

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
    world.register::<Texture>();
    world.register::<Velocity>();

    // Floor collision plane
    world.create_entity()
        .with(Position(nalgebra::Point3::new(-64.0, 5.0, -64.0)))
        .with(Collider::new(collider::ColliderType::HeightMap(
            heightmap_from_bmp("assets/terrain.bmp", 25.5).unwrap()
        )))
        .with(Model::new("assets/terrain.bmp"))
        .with(Texture::new("assets/sand.png"))
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
        .with(Texture::new("assets/marker.png"))
        .build();

    world.insert(ActiveCamera(Some(player)));
    world.insert(ActiveCharacter(Some(player)));

    world.create_entity()
        .with(Position(nalgebra::Point3::new(-8.0, -1.1, 16.0)))
        .with(Model {
            path: "assets/pillar.erm".to_string(),
            offset: Some(nalgebra::Vector3::new(0.0, 1.0, 0.0))
        })
        .with(Texture::new("assets/pillar.png"))
        .with(Collider::new(collider::ColliderType::Sphere(1.0)))
        .build();

    /*
    world.create_entity()
        .with(Position(nalgebra::Point3::new(-11.0, -0.8, 13.0)))
        .with(Model {
            path: "assets/elf-spear.erm".to_string(),
            offset: Some(nalgebra::Vector3::new(0.0, 0.5, 0.0))
        })
        .with(Collider::new(collider::ColliderType::Sphere(0.5)))
        .build();
    */

    world.create_entity()
        .with(Position(nalgebra::Point3::new(-14.0, -1.2, 10.0)))
        .with(Model {
            path: "assets/pillar.erm".to_string(),
            offset: Some(nalgebra::Vector3::new(0.0, 1.0, 0.0))
        })
        .with(Texture::new("assets/pillar.png"))
        .with(Collider::new(collider::ColliderType::Sphere(1.0)))
        .build();

    let dispatcher = DispatcherBuilder::new()
        .with(UpdateInputs, "update_inputs", &[])
        .with(PlayerMovement, "player_movement", &["update_inputs"])
        .with(Physics::new(&config.physics), "physics", &["player_movement"])
        .with(
            CollisionDetection::new(&config.physics),
            "collision_detection",
            &["physics"]
        )
        .with(
            CollisionResolver::new(&config.physics),
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