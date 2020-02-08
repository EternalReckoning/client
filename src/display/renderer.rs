use failure::{
    format_err,
    Error,
};
use rendy::init::AnyWindowedRendy;

use super::{
    DisplayConfig,
    RenderGraph,
    scene,
    ui::UI,
};

type Backend = rendy::vulkan::Backend;

pub struct Renderer {
    factory: rendy::factory::Factory<Backend>,
    families: rendy::command::Families<Backend>,
    scene: scene::Scene<Backend>,
    graph: Option<RenderGraph<Backend>>,
    _window: winit::window::Window,
}

impl Renderer {
    pub fn new(window: winit::window::WindowBuilder, event_loop: &winit::event_loop::EventLoop<()>, config: &DisplayConfig)
    -> Result<Renderer, Error>
    {
        let rendy_config: rendy::factory::Config = Default::default();

        let rendy = AnyWindowedRendy::init_auto(&rendy_config, window, &event_loop)
                .map_err(|err| {
                    log::error!("Graphics initialization failed: {}", err);
                    format_err!("failed to configure graphics device")
                })?;

        let time = std::time::Instant::now();

        Ok(rendy::with_any_windowed_rendy!((rendy)
            (mut factory, mut families, surface, window) => {
                let size = window.inner_size();
                let aspect = size.width as f32 / size.height as f32;
                
                let mut scene = scene::Scene {
                    camera: scene::Camera::new(aspect, config.field_of_view),
                    models: Vec::new(),
                    textures: Vec::new(),
                    objects: Vec::new(),
                    ticks: [time, time],
                    ui: UI::new(size.width as f64, size.height as f64),
                };

                let graph = Some(RenderGraph::new(
                    &mut factory,
                    &mut families,
                    &mut scene,
                    &size,
                    surface,
                ));

                Renderer { factory, families, scene, graph, _window: window }
            }
        ))
    }

    pub fn display(&mut self) {
        self.factory.maintain(&mut self.families);

        if let Some(graph) = &mut self.graph {
            graph.run(
                &mut self.factory,
                &mut self.families,
                &self.scene
            );
        }
    }

    pub fn get_scene(&mut self) -> &mut scene::Scene<Backend> {
        &mut self.scene
    }

    pub fn load_texture(&mut self, data: &crate::iohandler::FileLoaded)
        -> Result<(), Error>
    {
        if let Some(graph) = &mut self.graph {
            graph.load_mesh_texture(
                &mut self.factory,
                &mut self.scene,
                data
            )?;
            
            graph.load_ui_texture(
                &mut self.factory,
                &mut self.scene,
                data
            )?;
        }
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.scene.textures.clear();
        self.scene.ui.textures.clear();

        self.graph.take().unwrap().dispose(&mut self.factory, &self.scene);
    }
}