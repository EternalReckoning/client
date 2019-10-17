#[derive(Clone, Debug)]
pub struct Texture {
    pub path: String,
    pub wrap_mode: rendy::resource::WrapMode,
    pub format: Option<rendy::texture::image::ImageFormat>,
}

impl Texture {
    pub fn new(path: String) -> Texture {
        Texture {
            path,
            wrap_mode: rendy::resource::WrapMode::Clamp,
            format: None,
        }
    }
}