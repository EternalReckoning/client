use rendy::{
    factory::Factory,
    graph::render::{
        SimpleGraphicsPipeline,
        SimpleGraphicsPipelineDesc,
    },
    hal,
    hal::{
        adapter::PhysicalDevice,
        device::Device,
    },
};

use crate::display::scene::Scene;

lazy_static::lazy_static! {
    static ref VERTEX: rendy::shader::SpirvShader = rendy::shader::SourceShaderInfo::new(
        include_str!("mesh.vert"),
        "mesh.vert",
        rendy::shader::ShaderKind::Vertex,
        rendy::shader::SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();
    
    static ref FRAGMENT: rendy::shader::SpirvShader = rendy::shader::SourceShaderInfo::new(
        include_str!("mesh.frag"),
        "mesh.frag",
        rendy::shader::ShaderKind::Fragment,
        rendy::shader::SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SHADERS: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();
    
    static ref SHADER_REFLECTION: rendy::shader::SpirvReflection = SHADERS.reflect().unwrap();
}

const MAX_VERTEX_COUNT: usize = 524_288;
const MAX_INDEX_COUNT: usize = 2_097_152;
const MAX_OBJECT_COUNT: usize = 32;
const MAX_DESCRIPTOR_SET_COUNT: usize = 16;

const UNIFORM_SIZE: u64 = std::mem::size_of::<UniformArgs>() as u64;
const VERTEX_SIZE: u64 = (std::mem::size_of::<rendy::mesh::PosTex>() * MAX_VERTEX_COUNT) as u64;
const INDEX_SIZE: u64 = (std::mem::size_of::<u64>() * MAX_INDEX_COUNT) as u64;
const MODEL_SIZE: u64 = (std::mem::size_of::<InstanceArgs>() * MAX_OBJECT_COUNT) as u64;
const INDIRECT_COMMAND_SIZE: u64 = std::mem::size_of::<rendy::command::DrawIndexedCommand>() as u64;
const INDIRECT_SIZE: u64 = INDIRECT_COMMAND_SIZE * MAX_OBJECT_COUNT as u64;

const fn buffer_frame_size(size: u64, align: u64, index: usize) -> u64 {
    (((size - 1) / align + 1) * align) * index as u64
}

const fn uniform_offset(index: usize, align: u64) -> u64 {
    buffer_frame_size(UNIFORM_SIZE, align, index)
}

const fn vertex_offset(index: usize, align: u64, offset: u64) -> u64 {
    buffer_frame_size(VERTEX_SIZE, align, index) +
        (std::mem::size_of::<rendy::mesh::PosTex>() as u64 * offset)
}

const fn index_offset(index: usize, align: u64, offset: u64) -> u64 {
    buffer_frame_size(INDEX_SIZE, align, index) +
        (std::mem::size_of::<u32>() as u64 * offset)
}

const fn models_offset(index: usize, align: u64, offset: u64) -> u64 {
    buffer_frame_size(MODEL_SIZE, align, index) +
        (std::mem::size_of::<InstanceArgs>() as u64 * offset)
}

const fn indirect_offset(index: usize, align: u64, offset: u64) -> u64 {
    buffer_frame_size(INDIRECT_SIZE, align, index) +
        INDIRECT_COMMAND_SIZE * offset
}

#[derive(Copy, Clone, Debug)]
#[repr(C, align(16))]
struct UniformArgs {
    proj: nalgebra::Matrix4<f32>,
    view: nalgebra::Matrix4<f32>,
}

#[derive(Copy, Clone, Debug)]
#[repr(C, align(16))]
struct InstanceArgs {
    model: nalgebra::Transform3<f32>,
}

#[derive(Debug)]
struct DescriptorUsage {
    texture: Option<String>,
    loaded: bool,
}

#[derive(Debug, Default)]
pub struct TriangleRenderPipelineDesc;

#[derive(Debug)]
pub struct TriangleRenderPipeline<B: hal::Backend> {
    align: u64,
    uniform_buf: rendy::resource::Escape<rendy::resource::Buffer<B>>,
    vertex_buf: rendy::resource::Escape<rendy::resource::Buffer<B>>,
    index_buf: rendy::resource::Escape<rendy::resource::Buffer<B>>,
    model_buf: rendy::resource::Escape<rendy::resource::Buffer<B>>,
    indirect_buf: rendy::resource::Escape<rendy::resource::Buffer<B>>,
    sets: Vec<rendy::resource::Escape<rendy::resource::DescriptorSet<B>>>,
    set_usage: Vec<DescriptorUsage>,
    mesh_count: usize,
}

impl<B> SimpleGraphicsPipelineDesc<B, Scene<B>> for TriangleRenderPipelineDesc
where
    B: hal::Backend,
{
    type Pipeline = TriangleRenderPipeline<B>;

    fn load_shader_set(
        &self,
        factory: &mut Factory<B>,
        _scene: &Scene<B>,
    ) -> rendy::shader::ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn rasterizer(&self) -> hal::pso::Rasterizer {
        hal::pso::Rasterizer {
            polygon_mode: hal::pso::PolygonMode::Fill,
            cull_face: hal::pso::Face::BACK,
            front_face: hal::pso::FrontFace::Clockwise,
            depth_clamping: false,
            depth_bias: None,
            conservative: false,
        }
    }

    fn vertices(
        &self,
    ) -> Vec<(
        Vec<hal::pso::Element<hal::format::Format>>,
        hal::pso::ElemStride,
        hal::pso::VertexInputRate,
    )> {
        return vec![
            SHADER_REFLECTION
                .attributes(&["position", "uv"])
                .unwrap()
                .gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex),
            SHADER_REFLECTION
                .attributes_range(2..6)
                .unwrap()
                .gfx_vertex_input_desc(hal::pso::VertexInputRate::Instance(1)),
        ];
    }

    fn layout(&self) -> rendy::graph::render::Layout {
        rendy::graph::render::Layout {
            sets: vec![rendy::graph::render::SetLayout {
                bindings: vec![
                    hal::pso::DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: hal::pso::DescriptorType::UniformBuffer,
                        count: 1,
                        stage_flags: hal::pso::ShaderStageFlags::GRAPHICS,
                        immutable_samplers: false,
                    },
                    hal::pso::DescriptorSetLayoutBinding {
                        binding: 1,
                        ty: hal::pso::DescriptorType::SampledImage,
                        count: 1,
                        stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                    hal::pso::DescriptorSetLayoutBinding {
                        binding: 2,
                        ty: hal::pso::DescriptorType::Sampler,
                        count: 1,
                        stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                ],
            }],
            push_constants: Vec::new(),
        }
    }

    fn build<'a>(
        self,
        ctx: &rendy::graph::GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: rendy::command::QueueId,
        scene: &Scene<B>,
        buffers: Vec<rendy::graph::NodeBuffer>,
        images: Vec<rendy::graph::NodeImage>,
        set_layouts: &[rendy::resource::Handle<rendy::resource::DescriptorSetLayout<B>>],
    ) -> Result<TriangleRenderPipeline<B>, hal::pso::CreationError> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert_eq!(set_layouts.len(), 1);

        let frames = ctx.frames_in_flight as _;
        let align = factory
            .physical()
            .limits()
            .min_uniform_buffer_offset_alignment;

        let ubuf = factory
            .create_buffer(
                rendy::resource::BufferInfo {
                    size: buffer_frame_size(UNIFORM_SIZE, align, frames),
                    usage: hal::buffer::Usage::UNIFORM,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();
        let vbuf = factory
            .create_buffer(
                rendy::resource::BufferInfo {
                    size: buffer_frame_size(VERTEX_SIZE, align, frames),
                    usage: hal::buffer::Usage::VERTEX,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();
        let ibuf = factory
            .create_buffer(
                rendy::resource::BufferInfo {
                    size: buffer_frame_size(INDEX_SIZE, align, frames),
                    usage: hal::buffer::Usage::INDEX,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();
        let mbuf = factory
            .create_buffer(
                rendy::resource::BufferInfo {
                    size: buffer_frame_size(MODEL_SIZE, align, frames),
                    usage: hal::buffer::Usage::VERTEX,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();
        let icmdbuf = factory
            .create_buffer(
                rendy::resource::BufferInfo {
                    size: buffer_frame_size(INDIRECT_SIZE, align, frames),
                    usage: hal::buffer::Usage::INDIRECT,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();

        let mut set_usage = Vec::with_capacity(MAX_DESCRIPTOR_SET_COUNT * frames);
        let mut sets = Vec::new();
        for index in 0..frames {
            for _ in 0..MAX_DESCRIPTOR_SET_COUNT {
                let set = factory
                    .create_descriptor_set(set_layouts[0].clone())
                    .unwrap();

                unsafe {
                    factory.write_descriptor_sets(vec![
                        hal::pso::DescriptorSetWrite {
                            set: set.raw(),
                            binding: 0,
                            array_offset: 0,
                            descriptors: Some(hal::pso::Descriptor::Buffer(
                                ubuf.raw(),
                                Some(uniform_offset(index, align))
                                    ..Some(uniform_offset(index, align) + UNIFORM_SIZE),
                            )),
                        },
                    ]);
                }

                sets.push(set);
                set_usage.push(DescriptorUsage {
                    texture: None,
                    loaded: false,
                });
            }
        }

        let mut mesh_count = 0;
        for object in &scene.objects {
            if object.model.is_none() {
                continue;
            }

            let model = scene.models.get(object.model.unwrap()).unwrap();
            mesh_count += model.len();
        }

        Ok(TriangleRenderPipeline {
            align,
            uniform_buf: ubuf,
            vertex_buf: vbuf,
            index_buf: ibuf,
            model_buf: mbuf,
            indirect_buf: icmdbuf,
            sets,
            set_usage,
            mesh_count,
        })
    }
}

impl<B> SimpleGraphicsPipeline<B, Scene<B>> for TriangleRenderPipeline<B>
where
    B: hal::Backend,
{
    type Desc = TriangleRenderPipelineDesc;

    fn prepare(
        &mut self,
        factory: &rendy::factory::Factory<B>,
        _queue: rendy::command::QueueId,
        _set_layouts: &[rendy::resource::Handle<rendy::resource::DescriptorSetLayout<B>>],
        index: usize,
        scene: &Scene<B>,
    ) -> rendy::graph::render::PrepareResult {
        unsafe {
            factory
                .upload_visible_buffer(
                    &mut self.uniform_buf,
                    uniform_offset(index, self.align),
                    &[UniformArgs {
                        proj: scene.camera.proj.to_homogeneous(),
                        view: scene.camera.view.inverse().to_homogeneous(),
                    }]
                )
                .unwrap();
        }
        
        let mut textures_added = false;
        
        let min_descriptor = index * MAX_DESCRIPTOR_SET_COUNT;
        let max_descriptor = min_descriptor + MAX_DESCRIPTOR_SET_COUNT;

        for tex in scene.textures.iter() {
            if tex.data.is_none() {
                continue;
            }

            let mut found = false;
            for set in &self.set_usage[min_descriptor..max_descriptor] {
                if Some(&tex.path) == set.texture.as_ref() {
                    found = true;
                    break;
                }
            }
            
            if !found {
                for set_i in min_descriptor..max_descriptor {
                    let usage = self.set_usage.get_mut(set_i).unwrap();
                    if usage.texture.is_some() {
                        continue;
                    }

                    let set = self.sets.get(set_i).unwrap();

                    usage.texture = Some(tex.path.clone());
                    unsafe {
                        factory.write_descriptor_sets(vec![
                            hal::pso::DescriptorSetWrite {
                                set: set.raw(),
                                binding: 1,
                                array_offset: 0,
                                descriptors: Some(hal::pso::Descriptor::Image(
                                    tex.data.as_ref().unwrap().view().raw(),
                                    hal::image::Layout::ShaderReadOnlyOptimal,
                                )),
                            },
                            hal::pso::DescriptorSetWrite {
                                set: set.raw(),
                                binding: 2,
                                array_offset: 0,
                                descriptors: Some(hal::pso::Descriptor::Sampler(
                                    tex.data.as_ref().unwrap().sampler().raw()
                                )),
                            },
                        ]);
                    }
                    usage.loaded = true;

                    textures_added = true;

                    break;
                }
            }
        }

        let mut model_offsets = Vec::<(u32, u32, u32)>::new();

        let mut offset_v: u32 = 0;
        let mut offset_i: u32 = 0;
        for model_i in 0..scene.models.len() {
            let model = &scene.models[model_i];

            let mut index_count = 0;
            for mesh_i in 0..model.len() {
                index_count += model.get(mesh_i).unwrap().indices.len();
            }
            model_offsets.push((offset_v, offset_i, index_count as u32));

            for mesh_i in 0..model.len() {
                let mesh = model.get(mesh_i).unwrap();
                unsafe {
                    factory
                        .upload_visible_buffer(
                            &mut self.vertex_buf,
                            vertex_offset(index, self.align, offset_v as u64),
                            mesh.vertices.as_slice(),
                        )
                        .unwrap();

                    factory
                        .upload_visible_buffer(
                            &mut self.index_buf,
                            index_offset(index, self.align, offset_i as u64),
                            mesh.indices.as_slice(),
                        )
                        .unwrap();
                }

                offset_v += mesh.len();
                offset_i += mesh.indices.len() as u32;
            }
        }

        assert!(offset_v < MAX_VERTEX_COUNT as u32);
        assert!(offset_i < MAX_INDEX_COUNT as u32);

        let mut mesh_count = 0;
        for object_i in 0..scene.objects.len() {
            let object = &scene.objects[object_i];

            let model_id = match object.model {
                Some(model_id) => model_id,
                None => continue,
            };

            let model = scene.models.get(model_id).unwrap();
            mesh_count += model.len();
            let (offset_v, offset_i, index_count) =
                model_offsets.get(model_id).unwrap();

            unsafe {
                factory
                    .upload_visible_buffer(
                        &mut self.model_buf,
                        models_offset(index, self.align, object_i as u64),
                        &[InstanceArgs {
                            model: match model.offset {
                                Some(offset) => nalgebra::convert(object.position * nalgebra::Translation3::from(offset)),
                                None => nalgebra::convert(object.position),
                            }
                        }],
                    )
                    .unwrap();

                factory
                    .upload_visible_buffer(
                        &mut self.indirect_buf,
                        indirect_offset(index, self.align, object_i as u64),
                        &[rendy::command::DrawIndexedCommand {
                            index_count: *index_count,
                            instance_count: 1,
                            first_index: *offset_i,
                            vertex_offset: *offset_v as i32,
                            first_instance: object_i as u32,
                        }],
                    )
                    .unwrap();
            }
        }

        if textures_added || self.mesh_count != mesh_count {
            log::trace!("Vertices: {}; Indices: {}", offset_v, offset_i);
            rendy::graph::render::PrepareResult::DrawRecord
        } else {
            rendy::graph::render::PrepareResult::DrawReuse
        }
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: rendy::command::RenderPassEncoder<'_, B>,
        index: usize,
        scene: &Scene<B>,
    ) {
        unsafe {
            encoder.bind_vertex_buffers(
                0,
                std::iter::once((self.vertex_buf.raw(), vertex_offset(index, self.align, 0)))
            );

            encoder.bind_vertex_buffers(
                1,
                std::iter::once((self.model_buf.raw(), models_offset(index, self.align, 0)))
            );

            encoder.bind_index_buffer(
                self.index_buf.raw(),
                index_offset(index, self.align, 0),
                hal::IndexType::U32,
            );

            let min_descriptor = index * MAX_DESCRIPTOR_SET_COUNT;
            let max_descriptor = min_descriptor + MAX_DESCRIPTOR_SET_COUNT;

            for (tex_i, tex) in scene.textures.iter().enumerate() {
                for set_i in min_descriptor..max_descriptor {
                    let set = self.set_usage.get(set_i).unwrap();
                    if !set.loaded {
                        continue;
                    }
                    if set.texture.as_ref() != Some(&tex.path) {
                        continue;
                    }

                    encoder.bind_graphics_descriptor_sets(
                        layout,
                        0,
                        Some(self.sets[set_i].raw()),
                        std::iter::empty(),
                    );

                    for obj_i in 0..scene.objects.len() {
                        let obj = scene.objects.get(obj_i).unwrap();

                        if obj.texture != Some(tex_i) {
                            continue;
                        }

                        encoder.draw_indexed_indirect(
                            self.indirect_buf.raw(),
                            indirect_offset(index, self.align, obj_i as u64),
                            1,
                            INDIRECT_COMMAND_SIZE as u32,
                        );
                    }
                }
            }
        }
    }

    fn dispose(self, _factory: &mut rendy::factory::Factory<B>, _scene: &Scene<B>) {}
}
