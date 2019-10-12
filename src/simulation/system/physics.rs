use specs::prelude::*;

use crate::simulation::PhysicsConfig;
use crate::simulation::{
    component::{
        Position,
        Velocity,
    },
};

pub struct Physics {
    gravity: nalgebra::Vector3::<f64>,
    horisontal_drag_coeff: f64,
    vertical_drag_coeff: f64,
}

impl Physics {
    pub fn new(config: &PhysicsConfig) -> Physics {
        Physics {
            gravity: nalgebra::Vector3::new(0.0, config.gravity, 0.0),
            horisontal_drag_coeff: 1.0 - config.horisontal_drag,
            vertical_drag_coeff: 1.0 - config.vertical_drag,
        }
    }
}

impl<'a> System<'a> for Physics {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut pos, mut vel) = data;

        for (pos, vel) in (&mut pos, &mut vel).join() {
            pos.0 += vel.0;

            vel.0.x *= self.horisontal_drag_coeff;
            vel.0.y *= self.vertical_drag_coeff;
            vel.0.z *= self.horisontal_drag_coeff;

            vel.0 += self.gravity;
        }
    }
}