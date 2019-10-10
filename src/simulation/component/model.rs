use specs::prelude::*;

pub struct Model {
    pub path: String,
    pub offset: nalgebra::Vector3<f64>,
}

impl Component for Model {
    type Storage = VecStorage<Self>;
}