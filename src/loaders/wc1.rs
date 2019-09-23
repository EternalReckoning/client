use std::io::Read;

use bitflags::bitflags;

use crate::renderer::mesh::MeshBuilder;

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
struct Index(u64);

pub fn mesh_from_wc1(mut reader: std::io::BufReader<std::fs::File>) -> Result<MeshBuilder, failure::Error> {
    let mut header: Header = unsafe { std::mem::zeroed() };
    let header_size = std::mem::size_of::<Header>();

    unsafe {
        let header_slice = std::slice::from_raw_parts_mut(
            &mut header as *mut _ as *mut u8,
            header_size,
        );
        reader.read_exact(header_slice)?;
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let mut mesh_builder = MeshBuilder::new();

    let mut object_header: ObjectHeader = unsafe { std::mem::zeroed() };
    let object_header_size = std::mem::size_of::<ObjectHeader>();

    let mut vertex: Vertex = unsafe { std::mem::zeroed() };
    let vertex_size = std::mem::size_of::<Vertex>();

    let mut index: Index = unsafe { std::mem::zeroed() };
    let index_size = std::mem::size_of::<Index>();

    for _object_index in 0..header.object_count {
        unsafe {
            let object_header_slice = std::slice::from_raw_parts_mut(
                &mut object_header as *mut _ as *mut u8,
                object_header_size,
            );
            reader.read_exact(object_header_slice)?;
        }

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
            ]);
        }

        if object_header.flags.contains(ObjectFlags::HAS_COLORS) {
            let mut colors = Vec::with_capacity(object_header.index_count as usize);

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
                ]);
            }

            mesh_builder = mesh_builder.with_colors(&colors);
        }

        for _index_index in 0..object_header.index_count {
            unsafe {
                let index_slice = std::slice::from_raw_parts_mut(
                    &mut index as *mut _ as *mut u8,
                    index_size,
                );
                reader.read_exact(index_slice)?;
            }

            indices.push(index.0);
        }
    }

    let mut rearranged_vertices: Vec<[f32; 3]> = Vec::with_capacity(indices.len());
    for index in &indices {
        let index = (*index) as usize;
        let vertex: [f32; 3] = vertices.get(index).unwrap().clone();
        rearranged_vertices.push(vertex.clone());
    }

    Ok(mesh_builder.with_vertices(&rearranged_vertices))
}