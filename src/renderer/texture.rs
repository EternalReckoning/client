#[derive(Clone, Debug)]
pub struct Texture {
    pub path: String,
    pub wrap_mode: rendy::resource::WrapMode,
}

impl Texture {
    pub fn new(path: String) -> Texture {
        Texture {
            path,
            wrap_mode: rendy::resource::WrapMode::Clamp,
        }
    }
}