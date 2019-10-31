use rendy::hal;

#[derive(Debug)]
pub struct Texture<B>
where
    B: hal::Backend,
{
    pub path: String,
    pub wrap_mode: rendy::resource::WrapMode,
    pub format: Option<rendy::texture::image::ImageFormat>,
    pub data: Option<rendy::texture::Texture<B>>,
}

impl<B> Texture<B>
where
    B: hal::Backend,
{
    pub fn new(path: String) -> Texture<B> {
        Texture {
            path,
            wrap_mode: rendy::resource::WrapMode::Clamp,
            format: None,
            data: None,
        }
    }
}