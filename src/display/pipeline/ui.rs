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
        include_str!("ui.vert"),
        "ui.vert",
        rendy::shader::ShaderKind::Vertex,
        rendy::shader::SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();
    
    static ref FRAGMENT: rendy::shader::SpirvShader = rendy::shader::SourceShaderInfo::new(
        include_str!("ui.frag"),
        "ui.frag",
        rendy::shader::ShaderKind::Fragment,
        rendy::shader::SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SHADERS: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();
    
    static ref SHADER_REFLECTION: rendy::shader::SpirvReflection = SHADERS.reflect().unwrap();
}

const MAX_COMPONENT_COUNT: usize = 1024;
const MAX_VERTEX_COUNT: usize = MAX_COMPONENT_COUNT * 6;
const MAX_DESCRIPTOR_SET_COUNT: usize = 4;

const UNIFORM_SIZE: u64 = std::mem::size_of::<UniformArgs>() as u64;
const VERTEX_SIZE: u64 = (std::mem::size_of::<rendy::mesh::PosTex>() * MAX_VERTEX_COUNT) as u64;
const INDIRECT_COMMAND_SIZE: u64 = std::mem::size_of::<rendy::command::DrawCommand>() as u64;
const INDIRECT_SIZE: u64 = INDIRECT_COMMAND_SIZE * MAX_COMPONENT_COUNT as u64;


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

const fn indirect_offset(index: usize, align: u64, offset: u64) -> u64 {
    buffer_frame_size(INDIRECT_SIZE, align, index) +
        INDIRECT_COMMAND_SIZE * offset
}

#[derive(Copy, Clone, Debug)]
#[repr(C, align(16))]
struct UniformArgs {
    proj: nalgebra::Matrix4<f32>,
}

#[derive(Debug)]
struct DescriptorUsage {
    texture: Option<String>,
    loaded: bool,
}

#[derive(Debug, Default)]
pub struct SpriteGraphicsPipelineDesc;

#[derive(Debug)]
pub struct SpriteGraphicsPipeline<B: hal::Backend> {
    align: u64,
    uniform_buf: rendy::resource::Escape<rendy::resource::Buffer<B>>,
    vertex_buf: rendy::resource::Escape<rendy::resource::Buffer<B>>,
    indirect_buf: rendy::resource::Escape<rendy::resource::Buffer<B>>,
    sets: Vec<rendy::resource::Escape<rendy::resource::DescriptorSet<B>>>,
    set_usage: Vec<DescriptorUsage>,
    texture_components: Vec<Vec<u32>>,
    component_count: u32,
}

impl<B> SimpleGraphicsPipelineDesc<B, Scene<B>> for SpriteGraphicsPipelineDesc
where
    B: hal::Backend,
{
    type Pipeline = SpriteGraphicsPipeline<B>;

    fn depth_stencil(&self) -> Option<hal::pso::DepthStencilDesc> {
        None
    }

    fn load_shader_set(
        &self,
        factory: &mut Factory<B>,
        _scene: &Scene<B>,
    ) -> rendy::shader::ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn vertices(
        &self,
    ) -> Vec<(
        Vec<hal::pso::Element<hal::format::Format>>,
        hal::pso::ElemStride,
        hal::pso::VertexInputRate,
    )> {
        return vec![SHADER_REFLECTION
            .attributes(&["position", "uv"])
            .unwrap()
            .gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex)
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
        _scene: &Scene<B>,
        buffers: Vec<rendy::graph::NodeBuffer>,
        images: Vec<rendy::graph::NodeImage>,
        set_layouts: &[rendy::resource::Handle<rendy::resource::DescriptorSetLayout<B>>],
    ) -> Result<SpriteGraphicsPipeline<B>, hal::pso::CreationError> {
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

        Ok(SpriteGraphicsPipeline {
            align,
            sets,
            set_usage,
            texture_components: Vec::new(),
            component_count: 0,
            uniform_buf: ubuf,
            vertex_buf: vbuf,
            indirect_buf: icmdbuf,
        })
    }
}

impl<B> SimpleGraphicsPipeline<B, Scene<B>> for SpriteGraphicsPipeline<B>
where
    B: hal::Backend,
{
    type Desc = SpriteGraphicsPipelineDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
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
                        proj: scene.ui.proj.to_homogeneous(),
                    }]
                )
                .unwrap();
        }

        let mut textures_added = false;

        let min_descriptor = index * MAX_DESCRIPTOR_SET_COUNT;
        let max_descriptor = min_descriptor + MAX_DESCRIPTOR_SET_COUNT;

        for tex in scene.ui.textures.iter() {
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

        // FIXME: optimize away
        for texture_components in &mut self.texture_components {
            texture_components.clear();
        }

        let mut component_index = 0;
        for component in scene.ui.root.iter() {
            let component = match component.upgrade() {
                Some(component) => component,
                None => continue,
            };

            let mc = component.borrow();
            if mc.display.is_none() {
                continue;
            }

            let display = mc.display.as_ref().unwrap();

            for (tex_i, tex) in scene.ui.textures.iter().enumerate() {
                if tex_i >= self.texture_components.len() {
                    self.texture_components.push(Vec::new());
                }

                if &display.texture[..] == &tex.path[..] {
                    self.texture_components.get_mut(tex_i).unwrap()
                        .push(component_index as u32);
                }
            }

            unsafe {
                factory.upload_visible_buffer(
                    &mut self.vertex_buf,
                    vertex_offset(index, self.align, component_index * 6),
                    &[
                        rendy::mesh::PosTex {
                            position: [mc.dimensions.right as f32, mc.dimensions.top as f32, 0.0].into(),
                            tex_coord: [display.texture_coords.1[0], display.texture_coords.0[1]].into(),
                        },
                        rendy::mesh::PosTex {
                            position: [mc.dimensions.left as f32, mc.dimensions.bottom as f32, 0.0].into(),
                            tex_coord: [display.texture_coords.0[0], display.texture_coords.1[1]].into(),
                        },
                        rendy::mesh::PosTex {
                            position: [mc.dimensions.left as f32, mc.dimensions.top as f32, 0.0].into(),
                            tex_coord: [display.texture_coords.0[0], display.texture_coords.0[1]].into(),
                        },
                        rendy::mesh::PosTex {
                            position: [mc.dimensions.right as f32, mc.dimensions.top as f32, 0.0].into(),
                            tex_coord: [display.texture_coords.1[0], display.texture_coords.0[1]].into(),
                        },
                        rendy::mesh::PosTex {
                            position: [mc.dimensions.right as f32, mc.dimensions.bottom as f32, 0.0].into(),
                            tex_coord: [display.texture_coords.1[0], display.texture_coords.1[1]].into(),
                        },
                        rendy::mesh::PosTex {
                            position: [mc.dimensions.left as f32, mc.dimensions.bottom as f32, 0.0].into(),
                            tex_coord: [display.texture_coords.0[0], display.texture_coords.1[1]].into(),
                        },
                    ],
                )
                    .unwrap();
            }
            
            unsafe {
                factory.upload_visible_buffer(
                    &mut self.indirect_buf,
                    indirect_offset(index, self.align, component_index),
                    &[rendy::command::DrawCommand {
                        vertex_count: 6,
                        instance_count: 1,
                        first_vertex: component_index as u32 * 6,
                        first_instance: 0,
                    }]
                )
                .unwrap();
            }

            component_index += 1;
        }

        if textures_added || self.component_count != component_index as u32 {
            self.component_count = component_index as u32;
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

            let min_descriptor = index * MAX_DESCRIPTOR_SET_COUNT;
            let max_descriptor = min_descriptor + MAX_DESCRIPTOR_SET_COUNT;

            for (tex_i, tex) in scene.ui.textures.iter().enumerate() {
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

                    for component in self.texture_components.get(tex_i).unwrap() {
                        encoder.draw_indirect(
                            self.indirect_buf.raw(),
                            indirect_offset(index, self.align, *component as u64),
                            1,
                            INDIRECT_COMMAND_SIZE as u32
                        );
                    }

                    break;
                }
            }
        }
    }

    fn dispose(self, _factory: &mut rendy::factory::Factory<B>, _scene: &Scene<B>) {}
}