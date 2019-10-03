use specs::prelude::*;

use crate::input::MouseEuler;
use crate::simulation::{
    component::{
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
        ReadStorage<'a, Movement>,
        ReadStorage<'a, Jump>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (input, mouse_euler, mov, jump, mut pos, mut vel) = data;

        if input.move_up {
            for (jump, pos, vel) in (&jump, &pos, &mut vel).join() {
                if pos.0.y == 0.0 {
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