use specs::prelude::*;

use super::super::{
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
        ReadStorage<'a, Movement>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (input, mov, mut pos) = data;

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

        for (mov, pos) in (&mov, &mut pos).join() {
            pos.0 += movement * mov.speed;
        }
    }
}