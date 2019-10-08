use specs::prelude::*;

use crate::simulation::{
    component::{
        collider::{
            Collider,
            ColliderType,
            Collision,
        },
        Position,
    },
};

pub struct CollisionDetection;

impl<'a> System<'a> for CollisionDetection {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Collider>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, positions, mut colliders) = data;

        for collider in (&mut colliders).join() {
            collider.collisions.clear();
        }

        let mut collisions = Vec::new();

        // TODO: pre-checking with AABBs
        // TODO: fix the loop and avoid checking everything twice
        for (ent, pos, collider) in (&entities, &positions, &colliders).join() {
            for (target, target_pos, target_collider) in (&entities, &positions, &colliders).join() {
                if ent == target {
                    continue;
                }

                match check_collision(&pos, &collider.collider, &target_pos, &target_collider.collider) {
                    Some(depth) => {
                        collisions.push((
                            Collision { with: target, depth },
                            Collision { with: ent, depth: -depth }
                        ));
                    },
                    None => (),
                }
            }
        }

        for (c1, c2) in collisions {
            let e2 = c1.with;
            colliders.get_mut(c2.with).unwrap().collisions.push(c1);
            colliders.get_mut(e2).unwrap().collisions.push(c2);
        }
    }
}

fn check_collision(
    t1_pos: &Position, t1_collider: &ColliderType,
    t2_pos: &Position, t2_collider: &ColliderType,
) -> Option<nalgebra::Vector3<f64>>
{
    match t1_collider {
        ColliderType::Sphere(t1) => {
            match t2_collider {
                ColliderType::Sphere(t2) => {
                    sphere_to_sphere(&t1_pos, *t1, &t2_pos, *t2)
                },
                ColliderType::Plane(t2) => {
                    sphere_to_plane(&t1_pos, *t1, &t2_pos, t2)
                },
            }
        },
        ColliderType::Plane(_) => None,
    }
}

fn sphere_to_sphere(
    t1_pos: &Position, t1_radius: f64,
    t2_pos: &Position, t2_radius: f64,
) -> Option<nalgebra::Vector3<f64>>
{
    let min_distance = t1_radius + t2_radius;
    let collision_vec = t2_pos.0 - t1_pos.0;

    if collision_vec.x > min_distance || collision_vec.x < -min_distance {
        return None;
    }
    if collision_vec.y > min_distance || collision_vec.y < -min_distance {
        return None;
    }
    if collision_vec.z > min_distance || collision_vec.z < -min_distance {
        return None;
    }

    let collision_sq = nalgebra::distance_squared(&t1_pos.0, &t2_pos.0);
    if collision_sq <= min_distance*min_distance {
        let norm = collision_vec.normalize();
        return Some(norm * (min_distance - collision_sq.sqrt()));
    }

    None
}

fn sphere_to_plane(
    t1_pos: &Position, t1_radius: f64,
    t2_pos: &Position, t2_normal: &nalgebra::Unit<nalgebra::Vector3<f64>>,
) -> Option<nalgebra::Vector3<f64>>
{
    let collision_vec = t1_pos.0 - t2_pos.0;
    let distance = collision_vec.dot(t2_normal);

    if distance <= t1_radius {
        Some(t2_normal.as_ref() * (distance - t1_radius))
    } else {
        None
    }
}