use std::io::{
    BufReader,
    Cursor,
};

use rendy::{
    graph::render::{
        RenderGroupBuilder,
        SimpleGraphicsPipeline,
    },
    hal,
};
use failure::{
    format_err,
    Error,
};

use super::{
    pipeline::{
        mesh::TriangleRenderPipeline,
        ui::SpriteGraphicsPipeline,
    },
    scene::Scene,
};

pub struct RenderGraph<B: hal::Backend> {
    graph: rendy::graph::Graph<B, Scene<B>>,
    mesh_node: rendy::graph::NodeId,
    ui_node: rendy::graph::NodeId,
}

impl<B> RenderGraph<B>
where
    B: hal::Backend,
{
    pub fn new(
        mut factory: &mut rendy::factory::Factory<B>,
        mut families: &mut rendy::command::Families<B>,
        mut scene: &mut Scene<B>,
        window: &super::window::Window,
    ) -> RenderGraph<B> {
        let surface = window.create_surface(&mut factory).unwrap();

        let mut graph_builder = rendy::graph::GraphBuilder::<B, Scene<B>>::new();

        let win_size = window.get_size();
        let win_kind = hal::image::Kind::D2(win_size.width as u32, win_size.height as u32, 1, 1);

        let color = graph_builder.create_image(
            win_kind,
            1,
            factory.get_surface_format(&surface),
            Some(hal::command::ClearValue {
                color: hal::command::ClearColor {
                    float32: [1.0, 1.0, 1.0, 1.0],
                },
            }),
        );

        let depth = graph_builder.create_image(
            win_kind,
            1,
            hal::format::Format::D16Unorm,
            Some(hal::command::ClearValue {
                depth_stencil: hal::command::ClearDepthStencil {
                    depth: 1.0,
                    stencil: 0,
                },
            }),
        );

        let mesh_node = graph_builder.add_node(
            TriangleRenderPipeline::builder()
                .into_subpass()
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let ui_node = graph_builder.add_node(
            SpriteGraphicsPipeline::builder()
                .with_dependency(mesh_node)
                .into_subpass()
                .with_color(color)
                .into_pass(),
        );

        graph_builder.add_node(
            rendy::graph::present::PresentNode::builder(&factory, surface, color)
                .with_dependency(ui_node),
        );

        let graph = graph_builder
            .build(&mut factory, &mut families, &mut scene)
            .unwrap();

        RenderGraph { graph, mesh_node, ui_node }
    }

    pub fn run(
        &mut self,
        factory: &mut rendy::factory::Factory<B>,
        families: &mut rendy::command::Families<B>,
        scene: &Scene<B>,
    ) {
        self.graph.run(factory, families, &scene);
    }

    pub fn load_mesh_texture(
        &self,
        factory: &mut rendy::factory::Factory<B>,
        scene: &mut Scene<B>,
        data: &crate::iohandler::FileLoaded,
    ) -> Result<(), Error> {
        for texture in &mut scene.textures {
            if &texture.path == &data.path {
                if texture.data.is_some() {
                    // texture already loaded
                    return Ok(());
                }

                let queue = self.graph.node_queue(self.mesh_node);

                return self.load_texture(factory, queue, texture, data);
            }
        }

        // didn't need this texture...
        Ok(())
    }

    pub fn load_ui_texture(
        &self,
        factory: &mut rendy::factory::Factory<B>,
        scene: &mut Scene<B>,
        data: &crate::iohandler::FileLoaded,
    ) -> Result<(), Error> {
        for texture in &mut scene.ui.textures {
            if &texture.path == &data.path {
                if texture.data.is_some() {
                    // texture already loaded
                    return Ok(());
                }

                let queue = self.graph.node_queue(self.ui_node);

                return self.load_texture(factory, queue, texture, data);
            }
        }

        // didn't need this texture...
        Ok(())
    }

    fn load_texture(
        &self,
        factory: &mut rendy::factory::Factory<B>,
        queue: rendy::command::QueueId,
        texture: &mut super::Texture<B>,
        data: &crate::iohandler::FileLoaded,
    ) -> Result<(), Error> {
        let image_reader = BufReader::new(Cursor::new(&data.buf[..]));

        let mut texture_builder = rendy::texture::image::load_from_image(
            image_reader,
            rendy::texture::image::ImageTextureConfig {
                generate_mips: false,
                ..Default::default()
            }
        )?;

        let filter = rendy::resource::Filter::Linear;
        let data = texture_builder
            .set_sampler_info(
                hal::image::SamplerDesc {
                    min_filter: filter,
                    mag_filter: filter,
                    mip_filter: filter,
                    wrap_mode: (texture.wrap_mode, texture.wrap_mode, texture.wrap_mode),
                    lod_bias: hal::image::Lod::RANGE.start,
                    lod_range: hal::image::Lod::RANGE,
                    comparison: None,
                    border: rendy::resource::PackedColor(0),
                    normalized: true,
                    anisotropic: rendy::resource::Anisotropic::On(4),
                }
            )
            .build(
                rendy::factory::ImageState {
                    queue,
                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                    access: hal::image::Access::SHADER_READ,
                    layout: hal::image::Layout::ShaderReadOnlyOptimal,
                },
                factory,
            )
            .map_err(|e| {
                format_err!("Unable to load image to GPU: {} {:?}", &data.path, e)
            })?;
    
        texture.data = Some(data);

        Ok(())
    }

    pub fn dispose(
        self,
        factory: &mut rendy::factory::Factory<B>,
        scene: &Scene<B>,
    ) {
        self.graph.dispose(factory, &scene);
    }
}
