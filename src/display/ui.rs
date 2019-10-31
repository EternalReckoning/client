use rendy::hal;

use eternalreckoning_ui::{
    objectmodel::Tree,
    Component,
};

use super::component::Splash;

pub struct UI<B>
where
    B: hal::Backend,
{
    pub proj: nalgebra::Orthographic3<f32>,
    pub root: Tree,
    pub textures: Vec<super::Texture<B>>,
}

impl<B> UI<B>
where
    B: hal::Backend,
{
    pub fn new(width: f64, height: f64) -> UI<B> {
        UI {
            proj: nalgebra::Orthographic3::new(
                0.0,
                width as f32,
                0.0,
                height as f32,
                -1.0,
                1.0,
            ),
            root: Tree::new(width, height, Box::new(Splash::new())),
            textures: Vec::new(),
        }
    }

    pub fn set_root(&mut self, root: Box<dyn Component>) {
        self.root = Tree::new(self.root.width(), self.root.height(), root);
    }
}