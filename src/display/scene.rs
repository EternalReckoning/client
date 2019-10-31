use rendy::hal;

use crate::util::interpolate;
use super::ui::UI;

#[derive(Debug)]
pub struct Camera {
    pub view: nalgebra::Projective3<f32>,
    pub proj: nalgebra::Perspective3<f32>,
    pub position: nalgebra::Translation3<f32>,
    pub ticks: [Option<nalgebra::Point3<f32>>; 2],
}

#[derive(Debug)]
pub struct Object {
    pub id: specs::Entity,
    pub model: Option<usize>,
    pub texture: Option<usize>,
    pub position: nalgebra::Similarity3<f32>,
    pub ticks: [Option<nalgebra::Point3<f32>>; 2],
}

pub struct Scene<B>
where
    B: hal::Backend,
{
    pub camera: Camera,
    pub models: Vec<super::Model>,
    pub objects: Vec<Object>,
    pub textures: Vec<super::Texture<B>>,
    pub ticks: [std::time::Instant; 2],
    pub ui: UI<B>,
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
            position: nalgebra::Translation3::<f32>::new(0.0, 0.0, 0.0),
            ticks: [None, None],
        }
    }

    pub fn recalculate(&mut self, aspect: f32) {
        self.proj.set_aspect(aspect);
    }

    pub fn set_view(&mut self, view: nalgebra::Projective3<f32>) {
        self.view = view;
    }

    pub fn set_position(&mut self, position: nalgebra::Point3<f32>, interpolate: bool) {
        if interpolate {
            self.ticks[0] = self.ticks[1];
            self.ticks[1] = Some(position);
        } else {
            self.ticks[0] = None;
            self.ticks[1] = None;
            self.position = nalgebra::Translation3::<f32>::new(
                position.x,
                position.y,
                position.z
            )
        }
    }
}

impl Object {
    pub fn new(
        id: specs::Entity,
        position: nalgebra::Similarity3<f32>
    ) -> Object
    {
        Object {
            model: None,
            texture: None,
            ticks: [None, None],
            id,
            position,
        }
    }
}

impl<B> Scene<B>
where
    B: hal::Backend,
{
    pub fn interpolate_objects(&mut self, forward_interpolate: f32) {
        // this might be a bit dumb...
        let tick_ms = (self.ticks[1] - self.ticks[0]).subsec_millis();
        let elapsed = self.ticks[1].elapsed().subsec_millis();

        if tick_ms <= 0 || elapsed <= 0 {
            return;
        }

        let progress = elapsed as f32 / tick_ms as f32 + forward_interpolate;

        for object in &mut self.objects {
            if object.ticks[0].is_none() || object.ticks[1].is_none() {
                continue;
            }

            let interp_pos = interpolate::lerp(
                object.ticks[0].as_ref().unwrap(),
                object.ticks[1].as_ref().unwrap(),
                progress
            );

            object.position = nalgebra::Similarity3::<f32>::identity() *
                nalgebra::Translation3::<f32>::new(
                    interp_pos.x,
                    interp_pos.y,
                    interp_pos.z
                );
        }

        if self.camera.ticks[0].is_some() && self.camera.ticks[1].is_some() {
            let interp_pos = interpolate::lerp(
                self.camera.ticks[0].as_ref().unwrap(),
                self.camera.ticks[1].as_ref().unwrap(),
                progress
            );

            self.camera.position = nalgebra::Translation3::<f32>::new(
                interp_pos.x,
                interp_pos.y,
                interp_pos.z
            );
        }
    }

    pub fn set_model(
        &mut self,
        id: specs::Entity,
        path: &String,
        offset: Option<nalgebra::Vector3::<f32>>,
    ) -> bool
    {
        match self.object_by_id(id) {
            Some(index) => {
                let model = self.add_or_get_model(path, offset);
                let object = self.objects.get_mut(index).unwrap();
                object.model = Some(model);
                return true;
            },
            _ => false,
        }
    }

    pub fn set_texture(
        &mut self,
        id: specs::Entity,
        path: &String,
    ) -> bool
    {
        match self.object_by_id(id) {
            Some(index) => {
                let texture = self.get_texture(path);
                if texture.is_none() {
                    return false;
                }
                let texture = texture.unwrap();
                let object = self.objects.get_mut(index).unwrap();
                object.texture = Some(texture);
                return true;
            },
            _ => false,
        }
    }

    pub fn set_position(
        &mut self,
        id: specs::Entity,
        position: nalgebra::Point3::<f32>,
    ) -> bool
    {
        match self.object_by_id(id) {
            Some(index) => {
                let object = self.objects.get_mut(index).unwrap();
                object.ticks[0] = object.ticks[1];
                object.ticks[1] = Some(position);
                return true;
            },
            _ => false,
        }
    }

    pub fn get_model<'a>(
        &'a self,
        path: &str,
    ) -> Option<&'a super::Model>
    {
        for i in 0..self.models.len() {
            let model = self.models.get(i).unwrap();
            if &model.path == path {
                return Some(&model);
            }
        }
        None
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

    fn add_or_get_model(
        &mut self,
        path: &String,
        offset: Option<nalgebra::Vector3::<f32>>
    ) -> usize
    {
        for i in 0..self.models.len() {
            let model = self.models.get_mut(i).unwrap();

            if &model.path == path {
                if let Some(offset) = offset {
                    model.set_offset(offset);
                }
                return i;
            }
        }

        let mut model = super::Model::new(path.clone());
        if let Some(offset) = offset {
            model.set_offset(offset);
        }
        self.models.push(model);
        self.models.len() - 1
    }

    fn get_texture(
        &self,
        path: &String
    ) -> Option<usize>
    {
        for i in 0..self.textures.len() {
            let tex = self.textures.get(i).unwrap();

            if &tex.path == path {
                return Some(i);
            }
        }

        None
    }
    
    pub fn add_texture(
        &mut self,
        texture: super::Texture<B>
    ) -> usize
    {
        self.textures.push(texture);
        self.textures.len() - 1
    }
}