use failure::{
    Error,
    format_err,
};

use crate::renderer::{
    terrain::HeightMap,
    mesh::{
        Mesh,
        MeshBuilder,
    },
};

pub fn mesh_from_bmp(path: &str, scale: f32) -> Result<Mesh, Error> {
    let heightmap = heightmap_from_bmp(path, scale)?;

    let mesh_builder = MeshBuilder::new()
        .with_indices(&heightmap.indices())
        .with_vertices(&heightmap.vertices())
        .with_uvs(&heightmap.uvs());

    Ok(mesh_builder.build()?)
}

pub fn heightmap_from_bmp(path: &str, scale: f32) -> Result<HeightMap, Error>
{
    let mut reader = std::io::BufReader::new(
        std::fs::File::open(path)
            .map_err(|_| format_err!("cannot load heightmap: {}", path))?
    );
    let img = bmp::from_reader(&mut reader)?;

    if img.get_width() != img.get_height() {
        return Err(format_err!("heightmap is not square"));
    }

    let size = img.get_width() as usize;
    let mut data = Vec::<f32>::with_capacity(size * size);
    for (x, y) in img.coordinates() {
        let pixel = img.get_pixel(x, y);
        let value = (pixel.r as f32 + pixel.g as f32 + pixel.b as f32) / (3.0*255.0);
        data.push(value);
    }

    Ok(HeightMap::new(data, size, scale))
}