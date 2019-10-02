use specs::prelude::*;

use crate::simulation::{
    component::{
        Position,
        Velocity,
    },
};

pub struct Physics {
    gravity: nalgebra::Vector3::<f64>,
}

impl Physics {
    pub fn new(gravity: f64) -> Physics {
        Physics {
            gravity: nalgebra::Vector3::new(0.0, gravity, 0.0),
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

            if pos.0.y > 0.0 {
                pos.0.y = 0.0;
            }

            if pos.0.y < 0.0 {
                vel.0 += self.gravity;
            } else {
                vel.0.y = 0.0;
            }
        }
    }
}