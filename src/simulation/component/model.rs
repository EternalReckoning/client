use specs::prelude::*;

pub struct Model {
    pub path: String,
    pub offset: nalgebra::Vector3<f64>,
}

impl Component for Model {
    type Storage = VecStorage<Self>;
}

impl Model {
    pub fn new(path: &str) -> Model {
        Model {
            path: path.to_string(),
            offset: nalgebra::Vector3::new(0.0, 0.0, 0.0),
        }
    }
}