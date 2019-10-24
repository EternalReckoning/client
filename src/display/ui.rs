use eternalreckoning_ui::objectmodel::Tree;

use super::component::Hotbar;

pub struct UI {
    pub proj: nalgebra::Orthographic3<f32>,
    pub root: Tree,
}

impl UI {
    pub fn new(width: f64, height: f64) -> UI {
        UI {
            proj: nalgebra::Orthographic3::new(
                0.0,
                width as f32,
                0.0,
                height as f32,
                -1.0,
                1.0,
            ),
            root: Tree::new(width, height, Box::new(Hotbar::new())),
        }
    }
}