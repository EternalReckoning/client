use failure::Error;
use failure::format_err;

#[derive(Clone, Debug)]
pub struct Mesh {
    vertices: Vec<[f32; 3]>,
    colors: Option<Vec<[f32; 4]>>,
    pub indices: Option<Vec<u32>>,
}

pub struct MeshBuilder {
    vertices: Option<Vec<[f32; 3]>>,
    colors: Option<Vec<[f32; 4]>>,
    indices: Option<Vec<u32>>,
}

impl Mesh {
    pub fn len(&self) -> u32 {
        self.vertices.len() as u32
    }
}

impl std::convert::Into<Vec<rendy::mesh::PosColor>> for Mesh {
    fn into(self) -> Vec<rendy::mesh::PosColor> {
        let mut res = Vec::with_capacity(self.vertices.len());

        for index in 0..self.vertices.len() {
            let position = (self.vertices.get(index).unwrap()).clone().into();
            let color: _;

            match self.colors {
                None => {
                    color = [0.5, 0.5, 0.5, 1.0].into();
                },
                Some(ref colors) => {
                    color = (colors.get(index).unwrap()).clone().into();
                },
            }

            res.push(rendy::mesh::PosColor { position, color });
        }

        res
    }
}

impl MeshBuilder {
    pub fn new() -> MeshBuilder {
        MeshBuilder {
            vertices: None,
            colors: None,
            indices: None,
        }
    }

    pub fn with_vertices(
        mut self, vertices: &[[f32; 3]]
    ) -> MeshBuilder {
        let mut vert_vec = Vec::with_capacity(vertices.len());
        for vert in vertices {
            vert_vec.push(vert.clone());
        }
        self.vertices = Some(vert_vec);

        self
    }

    pub fn with_colors(
        mut self, colors: &[[f32; 4]]
    ) -> MeshBuilder {
        let mut col_vec = Vec::with_capacity(colors.len());
        for col in colors {
            col_vec.push(col.clone());
        }
        self.colors = Some(col_vec);

        self
    }

    pub fn with_indices(
        mut self, indices: &[u32]
    ) -> MeshBuilder {
        let mut index_vec = Vec::with_capacity(indices.len());
        for index in indices {
            index_vec.push(*index);
        }
        self.indices = Some(index_vec);

        self
    }


    pub fn build(self) -> Result<Mesh, Error> {
        if self.vertices.is_none() {
            return Err(format_err!("cannot build a mesh without vertex data"));
        }

        Ok(Mesh {
           vertices: self.vertices.unwrap(),
           colors: self.colors,
           indices: self.indices,
        })
    }
}