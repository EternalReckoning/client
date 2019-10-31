use failure::{
    format_err,
    Error,
};

use super::{
    DisplayConfig,
    RenderGraph,
    scene,
    ui::UI,
    window::Window,
};

type Backend = rendy::vulkan::Backend;

pub struct Renderer {
    factory: rendy::factory::Factory<Backend>,
    families: rendy::command::Families<Backend>,
    scene: scene::Scene<Backend>,
    graph: Option<RenderGraph<Backend>>,
}

impl Renderer {
    pub fn new(window: &Window, config: &DisplayConfig) -> Result<Renderer, Error> {
        let rendy_config: rendy::factory::Config = Default::default();
        let (mut factory, mut families): (rendy::factory::Factory<Backend>, _) =
            rendy::factory::init(rendy_config)
                .map_err(|err| format_err!("failed to configure graphics device: {:?}", err))?;

        let aspect = window.get_aspect_ratio() as f32;
        let size = window.get_size();
        let time = std::time::Instant::now();

        let mut scene = scene::Scene {
            camera: scene::Camera::new(aspect, config.field_of_view),
            models: Vec::new(),
            textures: Vec::new(),
            objects: Vec::new(),
            ticks: [time, time],
            ui: UI::new(size.width, size.height),
        };

        let graph = Some(RenderGraph::new(
            &mut factory,
            &mut families,
            &mut scene,
            &window,
        ));

        Ok(Renderer { factory, families, scene, graph })
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