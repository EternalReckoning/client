use specs::prelude::*;

use crate::simulation::{
    component::{
        collider::Collider,
        Position,
        Velocity,
    },
};

pub struct CollisionResolver;

impl<'a> System<'a> for CollisionResolver {
    type SystemData = (
        ReadStorage<'a, Collider>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (colliders, mut positions, mut velocities) = data;

        for (collider, pos, vel) in (&colliders, &mut positions, &mut velocities).join() {
            for collision in &collider.collisions {
                pos.0 -= collision.depth;

                match collision.depth.try_normalize(0.001) {
                    Some(normal) => {
                        vel.0 -= vel.0.dot(&normal) * normal;
                    },
                    _ => (),
                };
            }
        }
    }
}