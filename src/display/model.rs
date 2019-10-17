use super::Mesh;

#[derive(Clone, Debug)]
pub struct Model {
    pub path: String,
    pub offset: Option<nalgebra::Vector3<f32>>,
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
            offset: None,
            meshes: Vec::new(),
        }
    }

    pub fn set_offset(&mut self, offset: nalgebra::Vector3::<f32>) {
        self.offset = Some(offset);
    }
    
    pub fn len(&self) -> usize {
        self.meshes.len()
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