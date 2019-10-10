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
    pub id: specs::Entity,
    pub model: Option<usize>,
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
    pub fn new(aspect: f32, vfov: f32) -> Camera {
        Camera {
            proj: nalgebra::Perspective3::new(
                aspect,
                std::f32::consts::PI * (vfov / 180.0),
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

impl Scene {
    pub fn set_model(
        &mut self,
        id: specs::Entity,
        path: &String
    ) -> bool
    {
        match self.object_by_id(id) {
            Some(index) => {
                let model = self.add_or_get_model(path);
                let object = self.objects.get_mut(index).unwrap();
                object.model = Some(model);
                return true;
            },
            _ => false,
        }
    }

    pub fn set_position(
        &mut self,
        id: specs::Entity,
        position: nalgebra::Transform3::<f32>
    ) -> bool
    {
        match self.object_by_id(id) {
            Some(index) => {
                let object = self.objects.get_mut(index).unwrap();
                object.position = position;
                return true;
            },
            _ => false,
        }
    }

    fn object_by_id(&self, id: specs::Entity) -> Option<usize> {
        for i in 0..self.objects.len() {
            let object = self.objects.get(i).unwrap();

            if object.id == id {
                return Some(i);
            }
        }
        None
    }

    fn add_or_get_model(&mut self, path: &String) -> usize {
        for i in 0..self.models.len() {
            let model = self.models.get(i).unwrap();

            if &model.path == path {
                return i;
            }
        }

        self.models.push(super::Model::new(path.clone()));
        self.models.len() - 1
    }
}