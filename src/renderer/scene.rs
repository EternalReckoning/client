#[derive(Debug)]
pub struct Camera {
    pub view: nalgebra::Projective3<f32>,
    pub proj: nalgebra::Perspective3<f32>,
}

#[derive(Debug)]
pub struct UI {
    pub proj: nalgebra::Orthographic3<f32>,
}

#[derive(Debug)]
pub struct Object {
    pub model: u64,
    pub position: nalgebra::Transform3<f32>,
}

#[derive(Debug)]
pub struct Scene {
    pub camera: Camera,
    pub ui: UI,
    pub models: Vec<super::Model>,
    pub objects: Vec<Object>,
}

impl Camera {
    pub fn new(aspect: f32) -> Camera {
        Camera {
            proj: nalgebra::Perspective3::new(
                aspect,
                3.1415 / 4.0, // FOV in radians?
                1.0,
                200.0,
            ),
            view: nalgebra::Projective3::identity(),
        }
    }

    pub fn recalculate(&mut self, aspect: f32) {
        self.proj.set_aspect(aspect);
    }

    pub fn set_view(&mut self, view: nalgebra::Projective3<f32>) {
        self.view = view;
    }
}

impl UI {
    pub fn new(aspect: f32) -> UI {
        UI {
            proj: nalgebra::Orthographic3::new(
                -aspect,
                aspect,
                -1.0,
                1.0,
                -1.0,
                1.0,
            ),
        }
    }
}