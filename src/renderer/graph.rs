use rendy::{
    graph::render::{
        RenderGroupBuilder,
        SimpleGraphicsPipeline,
    },
    hal,
};

use super::scene::Scene;
use super::pipeline::mesh::TriangleRenderPipeline;
use super::pipeline::ui::SpriteGraphicsPipeline;

pub struct RenderGraph<B: hal::Backend> {
    graph: rendy::graph::Graph<B, Scene>,
}

impl<B> RenderGraph<B>
where
    B: hal::Backend,
{
    pub fn new(
        mut factory: &mut rendy::factory::Factory<B>,
        mut families: &mut rendy::command::Families<B>,
        mut scene: &mut Scene,
        window: &crate::window::Window,
    ) -> RenderGraph<B> {
        let surface = window.create_surface(&mut factory);

        let mut graph_builder = rendy::graph::GraphBuilder::<B, Scene>::new();

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

        let mesh_pass = graph_builder.add_node(
            TriangleRenderPipeline::builder()
                .into_subpass()
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let ui_pass = graph_builder.add_node(
            SpriteGraphicsPipeline::builder()
                .with_dependency(mesh_pass)
                .into_subpass()
                .with_color(color)
                .into_pass(),
        );

        graph_builder.add_node(
            rendy::graph::present::PresentNode::builder(&factory, surface, color).with_dependency(ui_pass),
        );

        let graph = graph_builder
            .build(&mut factory, &mut families, &mut scene)
            .unwrap();

        RenderGraph { graph }
    }

    pub fn run(
        &mut self,
        factory: &mut rendy::factory::Factory<B>,
        families: &mut rendy::command::Families<B>,
        scene: &Scene,
    ) {
        self.graph.run(factory, families, &scene);
    }

    pub fn dispose(
        self,
        factory: &mut rendy::factory::Factory<B>,
        scene: &Scene,
    ) {
        self.graph.dispose(factory, &scene);
    }
}
