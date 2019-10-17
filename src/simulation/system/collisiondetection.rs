use specs::prelude::*;

use crate::simulation::PhysicsConfig;
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
use crate::display::terrain::HeightMap;

pub struct CollisionDetection {
    min_collision_depth: f64,
}

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
            for (target, target_pos, target_collider)
                in (&entities, &positions, &colliders).join()
            {
                if ent == target {
                    continue;
                }

                match self.check_collision(
                    &pos,
                    &collider.collider,
                    &target_pos,
                    &target_collider.collider
                )
                {
                    Some((depth, normal)) => {
                        collisions.push((
                            Collision { with: target, depth, normal },
                            Collision { with: ent, depth: -depth, normal }
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

impl CollisionDetection {
    pub fn new(config: &PhysicsConfig) -> CollisionDetection {
        CollisionDetection { min_collision_depth: config.min_collision_depth }
    }

    fn check_collision(
        &self,
        t1_pos: &Position, t1_collider: &ColliderType,
        t2_pos: &Position, t2_collider: &ColliderType,
    ) -> Option<(nalgebra::Vector3<f64>, nalgebra::Unit<nalgebra::Vector3<f64>>)>
    {
        match t1_collider {
            ColliderType::Sphere(t1) => {
                match t2_collider {
                    ColliderType::Sphere(t2) => {
                        self.sphere_to_sphere(&t1_pos, *t1, &t2_pos, *t2)
                    },
                    ColliderType::Plane(t2) => {
                        self.sphere_to_plane(&t1_pos, *t1, &t2_pos, t2)
                    },
                    ColliderType::HeightMap(t2) => {
                        self.sphere_to_heightmap(&t1_pos, *t1, &t2_pos, t2)
                    }
                }
            },
            ColliderType::Plane(_) => None,
            ColliderType::HeightMap(_) => None,
        }
    }

    fn sphere_to_sphere(
        &self,
        t1_pos: &Position, t1_radius: f64,
        t2_pos: &Position, t2_radius: f64,
    ) -> Option<(nalgebra::Vector3<f64>, nalgebra::Unit<nalgebra::Vector3<f64>>)>
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
            if let Some(norm) = nalgebra::Unit::try_new(collision_vec, self.min_collision_depth) {
                return Some((
                    norm.as_ref() * (min_distance - collision_sq.sqrt()),
                    norm,
                ));
            }
        }

        None
    }

    fn sphere_to_plane(
        &self,
        t1_pos: &Position, t1_radius: f64,
        t2_pos: &Position, t2_normal: &nalgebra::Unit<nalgebra::Vector3<f64>>,
    ) -> Option<(nalgebra::Vector3<f64>, nalgebra::Unit<nalgebra::Vector3<f64>>)>
    {
        let collision_vec = t1_pos.0 - t2_pos.0;
        let distance = collision_vec.dot(t2_normal);

        let depth = t1_radius - distance;
        if depth >= self.min_collision_depth {
            let normal = nalgebra::Unit::new_unchecked(t2_normal.as_ref() * -1.0);
            Some((normal.as_ref() * depth, normal))
        } else {
            None
        }
    }

    fn sphere_to_heightmap(
        &self,
        t1_pos: &Position, t1_radius: f64,
        t2_pos: &Position, t2_data: &HeightMap,
    ) -> Option<(nalgebra::Vector3<f64>, nalgebra::Unit<nalgebra::Vector3<f64>>)>
    {
        // TODO: change to generic AABB check
        if t1_pos.0.x <= t2_pos.0.x ||
            t1_pos.0.x >= t2_pos.0.x + t2_data.size as f64 ||
            t1_pos.0.z <= t2_pos.0.z ||
            t1_pos.0.z >= t2_pos.0.z + t2_data.size as f64
        {
            return None
        }

        let map_offs = t1_pos.0 - t2_pos.0;
        let grid_x = map_offs.x as usize;
        let grid_y = map_offs.z as usize;

        let quad = (
            -t2_data.get(grid_x, grid_y).unwrap() as f64,
            -t2_data.get(grid_x + 1, grid_y).unwrap() as f64,
            -t2_data.get(grid_x, grid_y + 1).unwrap() as f64,
            -t2_data.get(grid_x + 1, grid_y + 1).unwrap() as f64,
        );
        let quad_x = map_offs.x - grid_x as f64;
        let quad_y = map_offs.y - grid_y as f64;

        // get the normal for the relevant height map triangle
        let normal: _;
        if quad_x + quad_y <= 0.5 {
            // cross(quad.1 - quad.0, quad.2 - quad.0)
            normal = nalgebra::Unit::new_normalize(
                nalgebra::Vector3::<f64>::new(1.0, quad.1 - quad.0, 0.0).cross(
                    &nalgebra::Vector3::<f64>::new(0.0, quad.2 - quad.0, 1.0)
                )
            );
        } else {
            // cross(quad.3 - quad.1, quad.2 - quad.1)
            normal = nalgebra::Unit::new_normalize(
                nalgebra::Vector3::<f64>::new(0.0, quad.3 - quad.1, 1.0).cross(
                    &nalgebra::Vector3::<f64>::new(-1.0, quad.2 - quad.1, 1.0)
                )
            );
        }

        let quad_pos = nalgebra::Point3::new(
            t2_pos.0.x + grid_x as f64,
            t2_pos.0.y + quad.0,
            t2_pos.0.z + grid_y as f64
        );
        let collision_vec = t1_pos.0 - quad_pos;
        let distance = collision_vec.dot(normal.as_ref());

        let depth = t1_radius - distance;
        if depth >= self.min_collision_depth {
            let normal = nalgebra::Unit::new_unchecked(normal.as_ref() * -1.0);
            Some((normal.as_ref() * depth, normal))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_to_heightmap_collision() {
        let detection = CollisionDetection { min_collision_depth: 0.001 };

        let heightmap_pos = Position(nalgebra::Point3::<f64>::new(0.0, 0.0, 0.0));
        let heightmap_data = vec![
            0.0, 0.0, 0.0,
            0.0, 0.0, 1.0,
            0.0, 1.0, 1.0,
        ];
        let heightmap = HeightMap::new(heightmap_data, 3, 1.0);

        let sphere_radius = 0.5;

        let sphere_pos = Position(nalgebra::Point3::<f64>::new(0.25, -0.5, 0.25));
        let result = detection.sphere_to_heightmap(
            &sphere_pos, sphere_radius,
            &heightmap_pos, &heightmap
        );
        assert!(result.is_none());

        let sphere_pos = Position(nalgebra::Point3::<f64>::new(0.25, 0.0, 0.25));
        let result = detection.sphere_to_heightmap(
            &sphere_pos, sphere_radius,
            &heightmap_pos, &heightmap
        );
        assert!(result.is_some());

        let (depth, normal) = result.unwrap();
        assert_eq!(depth.y, 0.5);
        assert_eq!(normal.y, 1.0);

        let sphere_pos = Position(nalgebra::Point3::<f64>::new(1.5, -0.5, 1.5));
        let result = detection.sphere_to_heightmap(
            &sphere_pos, sphere_radius,
            &heightmap_pos, &heightmap
        );
        assert!(result.is_some());
        
        let heightmap_pos = Position(nalgebra::Point3::<f64>::new(-10.0, 10.0, -10.0));
        let sphere_pos = Position(nalgebra::Point3::<f64>::new(-8.5, 9.5, -8.5));
        let result = detection.sphere_to_heightmap(
            &sphere_pos, sphere_radius,
            &heightmap_pos, &heightmap
        );
        assert!(result.is_some());
    }
}