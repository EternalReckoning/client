use specs::prelude::*;

pub struct Model {
    pub path: String,
    pub offset: Option<nalgebra::Vector3<f32>>,
}

impl Component for Model {
    type Storage = VecStorage<Self>;
}

impl Model {
    pub fn new(path: &str) -> Model {
        Model {
            path: path.to_string(),
            offset: None,
        }
    }
}