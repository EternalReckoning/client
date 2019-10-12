use failure::Error;
use failure::format_err;

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<rendy::mesh::PosTex>,
    pub indices: Vec<u32>,
}

pub struct MeshBuilder {
    vertices: Option<Vec<rendy::mesh::Position>>,
    colors: Option<Vec<rendy::mesh::Color>>,
    uvs: Option<Vec<rendy::mesh::TexCoord>>,
    indices: Option<Vec<u32>>,
}

impl Mesh {
    pub fn len(&self) -> u32 {
        self.vertices.len() as u32
    }
}

impl MeshBuilder {
    pub fn new() -> MeshBuilder {
        MeshBuilder {
            vertices: None,
            colors: None,
            uvs: None,
            indices: None,
        }
    }

    pub fn with_vertices(
        mut self, vertices: &[rendy::mesh::Position]
    ) -> MeshBuilder {
        let mut vert_vec = Vec::with_capacity(vertices.len());
        for vert in vertices {
            vert_vec.push(vert.clone());
        }
        self.vertices = Some(vert_vec);

        self
    }

    pub fn with_colors(
        mut self, colors: &[rendy::mesh::Color]
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

    pub fn with_uvs(
        mut self, uvs: &[rendy::mesh::TexCoord]
    ) -> MeshBuilder {
        let mut uv_vec = Vec::with_capacity(uvs.len());
        for uv in uvs {
            uv_vec.push(uv.clone());
        }
        self.uvs = Some(uv_vec);

        self
    }


    pub fn build(self) -> Result<Mesh, Error> {
        if self.vertices.is_none() {
            return Err(format_err!("cannot build a mesh without vertex data"));
        }
        if self.uvs.is_none() {
            return Err(format_err!("cannot build a mesh without UV data"));
        }
        if self.indices.is_none() {
            return Err(format_err!("cannot build a mesh without index data"));
        }

        let source = self.vertices.unwrap();
        let uv_source = self.uvs.unwrap();
        let mut vertices = Vec::with_capacity(source.len());
        for index in 0..source.len() {
            let position = source.get(index).unwrap().clone();
            let tex_coord = uv_source.get(index).unwrap().clone();

            vertices.push(rendy::mesh::PosTex { position, tex_coord });
        }

        Ok(Mesh {
           vertices,
           indices: self.indices.unwrap(),
        })
    }
}