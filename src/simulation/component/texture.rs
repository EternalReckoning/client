use specs::prelude::*;

pub struct Texture {
    pub path: String,
    pub wrap_mode: rendy::resource::WrapMode,
}

impl Component for Texture {
    type Storage = VecStorage<Self>;
}

impl Texture {
    pub fn new(path: &str) -> Texture {
        Texture {
            path: path.to_string(),
            wrap_mode: rendy::resource::WrapMode::Clamp,
        }
    }
}