use failure::Error;

use super::Mesh;
use crate::loaders::meshes_from_erm;

#[derive(Clone, Debug)]
pub struct Model {
    path: String,
    meshes: Vec<ModelMesh>,
}

#[derive(Clone, Debug)]
pub struct ModelMesh {
    position: nalgebra::Point3<f32>,
    mesh: Mesh,
}

impl Model {
    pub fn new(path: String) -> Model {
        Model {
            path,
            meshes: Vec::new(),
        }
    }
    
    pub fn len(&self) -> usize {
        self.meshes.len()
    }

    pub fn load(&mut self) -> Result<(), Error> {
        let meshes = meshes_from_erm(&self.path[..])?;
        for mesh in meshes {
            self.add_mesh(nalgebra::Point3::new(0.0, 0.0, 0.0), mesh);
        }
        Ok(())
    }
    
    pub fn get(&self, index: usize) -> Option<&Mesh> {
        match self.meshes.get(index) {
            Some(ref model_mesh) => Some(&model_mesh.mesh),
            None => None,
        }
    }

    pub fn add_mesh(&mut self, position: nalgebra::Point3<f32>, mesh: Mesh) {
        self.meshes.push(ModelMesh { position, mesh });
    }
}