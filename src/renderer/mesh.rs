#[derive(Clone, Debug)]
pub struct Mesh {
    vertices: Vec<[f32; 3]>,
    colors: Option<Vec<[f32; 4]>>,
}

pub struct MeshBuilder {
    vertices: Option<Vec<[f32; 3]>>,
    colors: Option<Vec<[f32; 4]>>,
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

    pub fn build(self) -> Result<Mesh, ()> {
        if self.vertices.is_none() {
            return Err(());
        }

        Ok(Mesh {
           vertices: self.vertices.unwrap(),
           colors: self.colors, 
        })
    }
}