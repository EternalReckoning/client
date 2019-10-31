use specs::prelude::*;

pub struct Terrain {
    pub path: String,
    pub offset: Option<nalgebra::Vector3<f32>>,
    pub scale: f32,
}

impl Component for Terrain {
    type Storage = VecStorage<Self>;
}

impl Terrain {
    pub fn new(path: &str, scale: f32) -> Terrain {
        Terrain {
            path: path.to_string(),
            offset: None,
            scale,
        }
    }
}