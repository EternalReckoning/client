use specs::prelude::*;

pub struct Velocity(pub nalgebra::Vector3<f64>);

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}