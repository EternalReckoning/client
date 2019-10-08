use specs::prelude::*;

use crate::input::MouseEuler;
use crate::simulation::{
    component::{
        Collider,
        Jump,
        Movement,
        Position,
        Velocity,
    },
    resource::InputMap,
};

pub struct PlayerMovement;

impl<'a> System<'a> for PlayerMovement {
    type SystemData = (
        Read<'a, InputMap>,
        Read<'a, MouseEuler>,
        ReadStorage<'a, Collider>,
        ReadStorage<'a, Movement>,
        ReadStorage<'a, Jump>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (input, mouse_euler, collider, mov, jump, mut pos, mut vel) = data;

        if input.move_up {
            for (collider, jump, vel) in (&collider, &jump, &mut vel).join() {
                let mut on_ground = false;
                let y_axis = nalgebra::Vector3::y();
                for collision in &collider.collisions {
                    if collision.depth.cross(&y_axis).magnitude_squared() <= 0.1 {
                        on_ground = true;
                        break;
                    }
                }

                if on_ground {
                    vel.0.y -= jump.force;
                }
            }
        }

        let mut movement = nalgebra::Vector3::<f64>::new(0.0, 0.0, 0.0);
        if input.move_forward {
            movement -= nalgebra::Vector3::z();
        }
        if input.move_backward {
            movement += nalgebra::Vector3::z();
        }
        if input.move_left {
            movement -= nalgebra::Vector3::x();
        }
        if input.move_right {
            movement += nalgebra::Vector3::x();
        }

        // normalizing <0, 0, 0> would produce <NaN, NaN, NaN>, we'd rather not...
        if movement.x != 0.0 || movement.y != 0.0 || movement.z != 0.0 {
            movement.normalize_mut();
            
            let rotation = nalgebra::Rotation3::from_axis_angle(
                &nalgebra::Vector3::<f64>::y_axis(),
                mouse_euler.yaw
            );

            movement = rotation.transform_vector(&movement);

            for (mov, pos) in (&mov, &mut pos).join() {
                pos.0 += movement * mov.speed;
            }
        }
    }
}