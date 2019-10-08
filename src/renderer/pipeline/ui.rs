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

use std::{fs::File, io::BufReader};

use crate::renderer::scene::Scene;

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

const VERTEX_COUNT: usize = 6;
const UNIFORM_SIZE: u64 = std::mem::size_of::<UniformArgs>() as u64;
const VERTEX_SIZE: u64 = (std::mem::size_of::<rendy::mesh::PosTex>() * VERTEX_COUNT) as u64;

const fn buffer_frame_size(align: u64) -> u64 {
    ((UNIFORM_SIZE + VERTEX_SIZE - 1) / align + 1) * align
}

const fn uniform_offset(index: usize, align: u64) -> u64 {
    buffer_frame_size(align) * index as u64
}

const fn vertex_offset(index: usize, align: u64) -> u64 {
    uniform_offset(index, align) + UNIFORM_SIZE
}

#[derive(Copy, Clone, Debug)]
#[repr(C, align(16))]
struct UniformArgs {
    proj: nalgebra::Matrix4<f32>,
}

#[derive(Debug, Default)]
pub struct SpriteGraphicsPipelineDesc;

#[derive(Debug)]
pub struct SpriteGraphicsPipeline<B: hal::Backend> {
    texture: rendy::texture::Texture<B>,
    align: u64,
    buffer: rendy::resource::Escape<rendy::resource::Buffer<B>>,
    sets: Vec<rendy::resource::Escape<rendy::resource::DescriptorSet<B>>>,
}

impl<B> SimpleGraphicsPipelineDesc<B, Scene> for SpriteGraphicsPipelineDesc
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
        _scene: &Scene,
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

    fn layout(&self) -> rendy::util::types::Layout {
        rendy::util::types::Layout {
            sets: vec![rendy::util::types::SetLayout {
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
        queue: rendy::command::QueueId,
        _scene: &Scene,
        buffers: Vec<rendy::graph::NodeBuffer>,
        images: Vec<rendy::graph::NodeImage>,
        set_layouts: &[rendy::resource::Handle<rendy::resource::DescriptorSetLayout<B>>],
    ) -> Result<SpriteGraphicsPipeline<B>, hal::pso::CreationError> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert_eq!(set_layouts.len(), 1);

        let image_reader = BufReader::new(
            File::open("assets/icon_attack.png")
            .map_err(|e| {
                log::error!("Unable to open {}: {:?}", "assets/icon_attack.png", e);
                hal::pso::CreationError::Other
            })?
        );

        let texture_builder = rendy::texture::image::load_from_image(
            image_reader,
            rendy::texture::image::ImageTextureConfig {
                generate_mips: true,
                ..Default::default()
            }
        ).map_err(|e| {
            log::error!("Unable to load image: {:?}", e);
            hal::pso::CreationError::Other
        })?;

        let texture = texture_builder
            .build(
                rendy::factory::ImageState {
                    queue,
                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                    access: hal::image::Access::SHADER_READ,
                    layout: hal::image::Layout::ShaderReadOnlyOptimal,
                },
                factory,
            )
            .unwrap();

        let frames = ctx.frames_in_flight as _;
        let align = factory
            .physical()
            .limits()
            .min_uniform_buffer_offset_alignment;

        let buffer = factory
            .create_buffer(
                rendy::resource::BufferInfo {
                    size: buffer_frame_size(align) * frames as u64,
                    usage: hal::buffer::Usage::VERTEX |
                        hal::buffer::Usage::UNIFORM,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();

        let mut sets = Vec::new();
        for index in 0..frames {
            let set = factory
                .create_descriptor_set(set_layouts[0].clone())
                .unwrap();

            unsafe {
                factory.device().write_descriptor_sets(vec![
                    hal::pso::DescriptorSetWrite {
                        set: set.raw(),
                        binding: 0,
                        array_offset: 0,
                        descriptors: vec![hal::pso::Descriptor::Buffer(
                            buffer.raw(),
                            Some(uniform_offset(index, align))
                                ..Some(uniform_offset(index, align) + UNIFORM_SIZE),
                        )],
                    },
                    hal::pso::DescriptorSetWrite {
                        set: set.raw(),
                        binding: 1,
                        array_offset: 0,
                        descriptors: vec![hal::pso::Descriptor::Image(
                            texture.view().raw(),
                            hal::image::Layout::ShaderReadOnlyOptimal,
                        )],
                    },
                    hal::pso::DescriptorSetWrite {
                        set: set.raw(),
                        binding: 2,
                        array_offset: 0,
                        descriptors: vec![hal::pso::Descriptor::Sampler(texture.sampler().raw())],
                    },
                ]);
            }

            sets.push(set);
        }

        Ok(SpriteGraphicsPipeline { texture, align, buffer, sets })
    }
}

impl<B> SimpleGraphicsPipeline<B, Scene> for SpriteGraphicsPipeline<B>
where
    B: hal::Backend,
{
    type Desc = SpriteGraphicsPipelineDesc;

    fn prepare(
        &mut self,
        factory: &rendy::factory::Factory<B>,
        _queue: rendy::command::QueueId,
        _set_layouts: &[rendy::resource::Handle<rendy::resource::DescriptorSetLayout<B>>],
        index: usize,
        scene: &Scene,
    ) -> rendy::graph::render::PrepareResult {
        unsafe {
            factory
                .upload_visible_buffer(
                    &mut self.buffer,
                    uniform_offset(index, self.align),
                    &[UniformArgs {
                        proj: scene.ui.proj.to_homogeneous(),
                    }]
                )
                .unwrap();
        }

        unsafe {
            factory.upload_visible_buffer(
                &mut self.buffer,
                vertex_offset(index, self.align),
                &[
                    rendy::mesh::PosTex {
                        position: [-0.05, 0.9, 0.0].into(),
                        tex_coord: [0.0, 1.0].into(),
                    },
                    rendy::mesh::PosTex {
                        position: [0.05, 0.9, 0.0].into(),
                        tex_coord: [1.0, 1.0].into(),
                    },
                    rendy::mesh::PosTex {
                        position: [0.05, 0.8, 0.0].into(),
                        tex_coord: [1.0, 0.0].into(),
                    },
                    rendy::mesh::PosTex {
                        position: [-0.05, 0.9, 0.0].into(),
                        tex_coord: [0.0, 1.0].into(),
                    },
                    rendy::mesh::PosTex {
                        position: [0.05, 0.8, 0.0].into(),
                        tex_coord: [1.0, 0.0].into(),
                    },
                    rendy::mesh::PosTex {
                        position: [-0.05, 0.8, 0.0].into(),
                        tex_coord: [0.0, 0.0].into(),
                    },
                ],
            )
                .unwrap();
        }

        rendy::graph::render::PrepareResult::DrawReuse
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: rendy::command::RenderPassEncoder<'_, B>,
        index: usize,
        _scene: &Scene,
    ) {
        unsafe {
            encoder.bind_graphics_descriptor_sets(
                layout,
                0,
                Some(self.sets[index].raw()),
                std::iter::empty::<u32>(),
            );

            encoder.bind_vertex_buffers(
                0,
                std::iter::once((self.buffer.raw(), vertex_offset(index, self.align)))
            );

            encoder.draw(0..(VERTEX_COUNT as u32), 0..1);
        }
    }

    fn dispose(self, _factory: &mut rendy::factory::Factory<B>, _scene: &Scene) {}
}
