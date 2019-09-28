use specs::prelude::*;

use crate::input::MouseEuler;
use crate::simulation::{
    component::{
        Movement,
        Position,
    },
    resource::InputMap,
};

pub struct PlayerMovement;

impl<'a> System<'a> for PlayerMovement {
    type SystemData = (
        Read<'a, InputMap>,
        Read<'a, MouseEuler>,
        ReadStorage<'a, Movement>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (input, mouse_euler, mov, mut pos) = data;

        if !input.move_forward && !input.move_backward && !input.move_left && !input.move_right {
            return;
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