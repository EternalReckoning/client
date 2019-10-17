use std::io::Read;

use bitflags::bitflags;
use failure::format_err;

use crate::display::{
    Mesh,
    mesh::MeshBuilder,
};

#[repr(C)]
struct Header {
    object_count: u64,
}

bitflags! {
    struct ObjectFlags: u64 {
        const HAS_COLORS = 0b00000001;
    }
}

#[repr(C)]
struct ObjectHeader {
    vertex_count: u64,
    index_count: u64,
    flags: ObjectFlags,
}

#[repr(C)]
struct Vertex {
    x: f64,
    y: f64,
    z: f64,
}

#[repr(C)]
struct VertexColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

#[repr(C)]
struct UV {
    u: f64,
    v: f64,
}

#[repr(C)]
struct Index(u64);

pub fn meshes_from_erm(path: &str) -> Result<Vec<Mesh>, failure::Error> {
    let mut reader = std::io::BufReader::new(
        std::fs::File::open(path)
            .map_err(|_| format_err!("cannot load model: {}", path))?
    );

    let mut header: Header = unsafe { std::mem::zeroed() };
    let header_size = std::mem::size_of::<Header>();

    unsafe {
        let header_slice = std::slice::from_raw_parts_mut(
            &mut header as *mut _ as *mut u8,
            header_size,
        );
        reader.read_exact(header_slice)?;
    }

    let mut meshes = Vec::new();
    let mut index_offset = 0;
    for _object_index in 0..header.object_count {
        let mesh = mesh_from_erm(&mut reader, index_offset)?;
        index_offset += mesh.len();
        meshes.push(mesh);
    }

    Ok(meshes)
}

fn mesh_from_erm(reader: &mut std::io::BufReader<std::fs::File>, index_offset: u32)
    -> Result<Mesh, failure::Error>
{
    let mut mesh_builder = MeshBuilder::new();

    let mut object_header: ObjectHeader = unsafe { std::mem::zeroed() };
    let object_header_size = std::mem::size_of::<ObjectHeader>();

    let mut vertex: Vertex = unsafe { std::mem::zeroed() };
    let vertex_size = std::mem::size_of::<Vertex>();

    let mut index: Index = unsafe { std::mem::zeroed() };
    let index_size = std::mem::size_of::<Index>();

    let mut uv: UV = unsafe { std::mem::zeroed() };
    let uv_size = std::mem::size_of::<UV>();

    unsafe {
        let object_header_slice = std::slice::from_raw_parts_mut(
            &mut object_header as *mut _ as *mut u8,
            object_header_size,
        );
        reader.read_exact(object_header_slice)?;
    }

    let mut vertices = Vec::with_capacity(object_header.vertex_count as usize);
    let mut colors = Vec::<rendy::mesh::Color>::with_capacity(object_header.index_count as usize);
    let mut indices = Vec::with_capacity(object_header.index_count as usize);
    let mut uvs: Vec<rendy::mesh::TexCoord> = Vec::with_capacity(object_header.index_count as usize);

    for _vertex_index in 0..object_header.vertex_count {
        unsafe {
            let vertex_slice = std::slice::from_raw_parts_mut(
                &mut vertex as *mut _ as *mut u8,
                vertex_size,
            );
            reader.read_exact(vertex_slice)?;
        }

        vertices.push([
            vertex.x as f32,
            vertex.y as f32,
            vertex.z as f32,
        ].into());
    }

    if object_header.flags.contains(ObjectFlags::HAS_COLORS) {
        let mut color: VertexColor = unsafe { std::mem::zeroed() };
        let color_size = std::mem::size_of::<VertexColor>();
        for _index in 0..object_header.index_count {
            unsafe {
                let color_slice = std::slice::from_raw_parts_mut(
                    &mut color as *mut _ as *mut u8,
                    color_size,
                );
                reader.read_exact(color_slice)?;
            }

            colors.push([
                color.r as f32,
                color.g as f32,
                color.b as f32,
                color.a as f32,
            ].into());
        }
    }

    for _index_index in 0..object_header.index_count {
        unsafe {
            let index_slice = std::slice::from_raw_parts_mut(
                &mut index as *mut _ as *mut u8,
                index_size,
            );
            reader.read_exact(index_slice)?;
        }

        indices.push(index.0 as u32);
    }

    for _uv_index in 0..object_header.index_count {
        unsafe {
            let uv_slice = std::slice::from_raw_parts_mut(
                &mut uv as *mut _ as *mut u8,
                uv_size,
            );
            reader.read_exact(uv_slice)?;
        }

        // TODO
        // UVs need to be flipped, probably due to vertex Y-axis flipping in export script.
        // The export script should be fixed instead...
        let texcoord: rendy::mesh::TexCoord = [uv.u as f32, 1.0 - uv.v as f32].into();
        uvs.push(texcoord);
    }

    if object_header.flags.contains(ObjectFlags::HAS_COLORS) {
        let mut reorder_colors = Vec::with_capacity(object_header.vertex_count as usize);
        for index in 0..(object_header.vertex_count as u32) {
            for index_index in 0..indices.len() {
                if *indices.get(index_index).unwrap() == index + index_offset {
                    reorder_colors.push(colors.get(index_index).unwrap().clone());
                    break;
                }
            }
        }
        assert_eq!(reorder_colors.len(), vertices.len());
        mesh_builder = mesh_builder.with_colors(&reorder_colors);
    }

    let (vertices, indices) = unroll_verts(vertices, indices);
    
    mesh_builder = mesh_builder.with_vertices(&vertices);
    mesh_builder = mesh_builder.with_indices(&indices);
    mesh_builder = mesh_builder.with_uvs(&uvs);

    Ok(mesh_builder.build()?)
}

fn unroll_verts(
    vertices: Vec::<rendy::mesh::Position>,
    mut indices: Vec::<u32>,
) -> (Vec::<rendy::mesh::Position>, Vec::<u32>)
{
    let mut reorder_verts = Vec::with_capacity(indices.len());
    for index in 0..(indices.len()) {
        let vert_index = indices.get_mut(index).unwrap();
        reorder_verts.push(*vertices.get(*vert_index as usize).unwrap());
        *vert_index = index as u32;
    }
    (reorder_verts, indices)
}