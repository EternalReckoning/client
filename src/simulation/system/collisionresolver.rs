use specs::prelude::*;

use crate::simulation::PhysicsConfig;
use crate::simulation::{
    component::{
        collider::Collider,
        Position,
        Velocity,
        Movement,
    },
};

pub struct CollisionResolver {
    min_ground_y: f64,
}

impl CollisionResolver {
    /**
     * 'max_ground_slope' gives the maximum slope in percentage notation
     * upon which something is considered to be "on ground"
     */
    pub fn new(config: &PhysicsConfig) -> CollisionResolver {
        CollisionResolver {
            min_ground_y: 1.0 - config.max_ground_slope,
        }
    }
}

impl<'a> System<'a> for CollisionResolver {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Collider>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, Movement>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entity, colliders, mut positions, mut velocities, mut movement) = data;

        for (ent, collider, pos, vel)
            in (&entity, &colliders, &mut positions, &mut velocities).join()
        {
            let mut on_ground = false;

            for collision in &collider.collisions {
                pos.0 -= collision.depth;
                vel.0 -= vel.0.dot(collision.normal.as_ref()) * collision.normal.as_ref();

                if collision.normal.as_ref().y >= self.min_ground_y {
                    on_ground = true;
                }
            }
            
            if let Some(mov) = movement.get_mut(ent) {
                mov.on_ground = on_ground;
            }
        }
    }
}