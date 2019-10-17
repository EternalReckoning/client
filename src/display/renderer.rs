use failure::{
    format_err,
    Error,
};

use crate::loaders;
use super::{
    DisplayConfig,
    Model,
    RenderGraph,
    scene,
    Texture,
    window::Window,
};

type Backend = rendy::vulkan::Backend;

pub struct Renderer {
    factory: rendy::factory::Factory<Backend>,
    families: rendy::command::Families<Backend>,
    scene: scene::Scene,
    graph: Option<RenderGraph<Backend>>,
}

impl Renderer {
    pub fn new(window: &Window, config: &DisplayConfig) -> Result<Renderer, Error> {
        let rendy_config: rendy::factory::Config = Default::default();
        let (mut factory, mut families): (rendy::factory::Factory<Backend>, _) =
            rendy::factory::init(rendy_config)
                .map_err(|err| format_err!("failed to configure graphics device: {:?}", err))?;

        let mut terrain = Model::new("assets/terrain.bmp".to_string());
        let terrain_mesh = loaders::mesh_from_bmp("assets/terrain.bmp", 25.0)?;
        terrain.add_mesh(
            nalgebra::Point3::new(0.0, 0.0, 0.0),
            terrain_mesh
        );

        let aspect = window.get_aspect_ratio() as f32;

        let mut scene = scene::Scene {
            camera: scene::Camera::new(aspect, config.field_of_view),
            ui: scene::UI::new(aspect),
            models: vec![terrain],
            // TODO: remove the need to hardcode textures here
            textures: vec![
                Texture {
                    path: "assets/stone.png".to_string(),
                    wrap_mode: rendy::resource::WrapMode::Tile,
                    format: None,
                },
                Texture {
                    path: "assets/marker.png".to_string(),
                    wrap_mode: rendy::resource::WrapMode::Clamp,
                    format: None,
                },
                Texture {
                    path: "assets/pillar.png".to_string(),
                    wrap_mode: rendy::resource::WrapMode::Clamp,
                    format: None,
                },
            ],
            objects: Vec::new(),
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

    pub fn get_scene(&mut self) -> &mut scene::Scene {
        &mut self.scene
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.graph.take().unwrap().dispose(&mut self.factory, &self.scene);
    }
}