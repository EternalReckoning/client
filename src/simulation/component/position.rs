use specs::prelude::*;

pub struct Position(pub nalgebra::Point3<f64>);

impl Component for Position {
    type Storage = VecStorage<Self>;
}