use std::sync::mpsc::{
    Sender,
    Receiver,
};

use failure::Error;

use crate::{
    input,
    input::InputTypes,
    iohandler,
    display::{
        self,
        Renderer,
        scene::{
            Scene,
            Object,
        },
    },
    simulation::event,
    util::config,
};

pub fn run(
    renderer: Renderer,
    event_loop: winit::event_loop::EventLoop<()>,
    config: config::Config,
    event_tx: Sender<event::Event>,
    update_rx: Receiver<event::Update>,
    io_channel: (Sender<iohandler::Request>, Receiver<iohandler::Response>),
) -> Result<(), Error> {
    let mut key_map = std::collections::HashMap::<u32, InputTypes>::new();
    key_map.insert(config.key_map.move_forward, InputTypes::MoveForward);
    key_map.insert(config.key_map.move_left, InputTypes::MoveLeft);
    key_map.insert(config.key_map.move_backward, InputTypes::MoveBackward);
    key_map.insert(config.key_map.move_right, InputTypes::MoveRight);
    key_map.insert(config.key_map.move_up, InputTypes::MoveUp);

    let (io_tx, io_rx) = io_channel;
    let mut renderer = Some(renderer);

    let mut loading = 0;

    let mouse_sens = input::MouseSensitivity::new(config.mouse.sensitivity);
    let mut mouse_euler = input::MouseEuler::default();
    let mut mouse_look = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                },
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    log::trace!(
                        "Keyboard input: {} {}",
                        input.scancode,
                        match input.state {
                            winit::event::ElementState::Pressed => "pressed",
                            winit::event::ElementState::Released => "released",
                        }
                    );

                    if input.scancode == 1 {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }

                    if let Some(action) = key_map.get(&input.scancode) {
                        let event = match input.state {
                            winit::event::ElementState::Pressed => {
                                event::InputEvent::KeyDown(*action)
                            },
                            winit::event::ElementState::Released => {
                                event::InputEvent::KeyUp(*action)
                            },
                        };
                        event_tx.send(event::Event::InputEvent(event)).unwrap();
                    }
                },
                winit::event::WindowEvent::MouseInput { button, state, .. } => {
                    log::trace!(
                        "Mouse input: {:?} button {}",
                        button,
                        match state {
                            winit::event::ElementState::Pressed => "pressed",
                            winit::event::ElementState::Released => "released",
                        },
                    );

                    if button == winit::event::MouseButton::Right {
                        mouse_look = state == winit::event::ElementState::Pressed;
                    }
                },
                _ => {},
            },
            winit::event::Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    if mouse_look {
                        mouse_euler.update(delta, &mouse_sens);
                        let event = event::InputEvent::CameraAngle(mouse_euler.clone());
                        event_tx.send(event::Event::InputEvent(event)).unwrap();
                    }
                },
                _ => {},
            },
            winit::event::Event::MainEventsCleared => {
                if let Some(renderer) = &mut renderer {
                    let scene = renderer.get_scene();

                    loop {
                        let update = update_rx.try_recv();
                        match update {
                            Ok(e) => {
                                match e {
                                    event::Update::PositionUpdate(event::PositionUpdate { entity, position, .. }) => {
                                        let position = nalgebra::Point3::new(
                                                position.x as f32,
                                                position.y as f32,
                                                position.z as f32
                                            );

                                        if !scene.set_position(entity, position.clone()) {
                                            scene.objects.push(Object::new(
                                                entity,
                                                nalgebra::Similarity3::<f32>::identity() *
                                                    nalgebra::Translation3::<f32>::new(
                                                        position.x,
                                                        position.y,
                                                        position.z
                                                    )
                                            ));
                                        }
                                    },
                                    event::Update::CameraUpdate(event::CameraUpdate(position)) => {
                                        scene.camera.set_position(
                                            nalgebra::Point3::<f32>::new(
                                                position.x as f32,
                                                position.y as f32,
                                                position.z as f32
                                            ),
                                            true
                                        );
                                    },
                                    event::Update::ModelUpdate(event::ModelUpdate { entity, ref path, offset }) => {
                                        if scene.get_model(&path[..]).is_none() {
                                            loading += 1;
                                            io_tx.send(iohandler::Request::LoadModel(path.to_string()))
                                                .unwrap_or_else(|err| {
                                                    log::error!("IO handler not available: {}", err);
                                                    *control_flow = winit::event_loop::ControlFlow::Exit;
                                                });
                                        }
                                        scene.set_model(entity, path, offset);
                                    },
                                    event::Update::TerrainUpdate(event::TerrainUpdate { entity, ref heightmap, scale }) => {
                                        if scene.get_model(&heightmap[..]).is_none() {
                                            loading += 1;
                                            io_tx.send(iohandler::Request::LoadTerrain(
                                                iohandler::LoadTerrainRequest {
                                                    path: heightmap.to_string(),
                                                    scale,
                                                }
                                            ))
                                                .unwrap_or_else(|err| {
                                                    log::error!("IO handler not available: {}", err);
                                                    *control_flow = winit::event_loop::ControlFlow::Exit;
                                                });
                                        }
                                        scene.set_model(entity, heightmap, None);
                                    },
                                    event::Update::TextureUpdate(event::TextureUpdate { entity, ref path, wrap_mode }) => {
                                        if !scene.set_texture(entity, path) {
                                            scene.add_texture(display::Texture {
                                                path: path.clone(),
                                                wrap_mode,
                                                format: None,
                                                data: None,
                                            });

                                            loading += 1;
                                            io_tx.send(iohandler::Request::LoadFile(path.to_string()))
                                                .unwrap_or_else(|err| {
                                                    log::error!("IO handler not available: {}", err);
                                                    *control_flow = winit::event_loop::ControlFlow::Exit;
                                                });
                                        }
                                    },
                                    event::Update::SimulationTick(time) => {
                                        scene.ticks[0] = scene.ticks[1];
                                        scene.ticks[1] = time;
                                    },
                                };
                            },
                            Err(_) => break,
                        }
                    }

                    scene.interpolate_objects(config.display.forward_interpolate);

                    let rotation = nalgebra::Rotation3::from_euler_angles(
                        mouse_euler.pitch as f32,
                        mouse_euler.yaw as f32,
                        0.0,
                    );
                    let translation = nalgebra::Translation3::<f32>::new(0.0, 0.0, 10.0);
                    scene.camera.set_view(
                        nalgebra::Projective3::identity() * scene.camera.position * rotation * translation
                    );

                    renderer.display();
                }
            },
            _ => {},
        }

        if let Some(renderer) = &mut renderer {
            let scene = renderer.get_scene();
            send_ui_texture_requests(scene, &io_tx);
        }

        loop {
            match io_rx.try_recv() {
                Ok(io_result) => {
                    match io_result {
                        iohandler::Response::TerrainLoaded(data) => {
                            if let Some(renderer) = &mut renderer {
                                terrain_loaded(renderer, data);
                            }
                        },
                        iohandler::Response::ModelLoaded(data) => {
                            if let Some(renderer) = &mut renderer {
                                model_loaded(renderer, data);
                            }
                        },
                        iohandler::Response::FileLoaded(data) => {
                            if let Some(renderer) = &mut renderer {
                                file_loaded(renderer, data);
                            }
                        },
                        // TODO
                        iohandler::Response::Error => (),
                    }
                    
                    loading -= 1;
                    if loading == 0 {
                        if let Some(renderer) = &mut renderer {
                            let scene = renderer.get_scene();
                            // loading splash screen hack
                            scene.ui.set_root(Box::new(display::component::Hotbar::new()));
                        }
                    }
                },
                Err(_) => break,
            };
        }

        if *control_flow == winit::event_loop::ControlFlow::Exit && renderer.is_some() {
            log::info!("Exiting...");
            renderer.take();
        }
    });
}

fn send_ui_texture_requests<B: rendy::hal::Backend>(
    scene: &mut Scene<B>,
    io_tx: &Sender<iohandler::Request>
) {
    for texture in scene.ui.root.iter_textures() {
        if let Some(texture) = texture.upgrade() {
            let mut found = false;
            for ui_texture in &scene.ui.textures {
                if &ui_texture.path[..] == &texture[..] {
                    found = true;
                    break;
                }
            }

            if !found {
                scene.ui.textures.push(display::Texture {
                    path: texture.to_string(),
                    wrap_mode: rendy::resource::WrapMode::Clamp,
                    format: None,
                    data: None,
                });
                io_tx.send(iohandler::Request::LoadFile(texture.to_string()))
                    .unwrap_or_else(|e| {
                        log::warn!("Failed to send IO request: {}", e);
                    });
            }
        }
    }
}

fn file_loaded(renderer: &mut Renderer, data: iohandler::FileLoaded) {
    renderer.load_texture(&data)
        .unwrap_or_else(|e| {
            log::error!("Failed to load texture: {}", e);
        });
}

fn model_loaded(renderer: &mut Renderer, data: iohandler::ModelLoaded) {
    let mut meshes = data.meshes;
    for model in &mut renderer.get_scene().models {
        if model.path == data.path {
            loop {
                match meshes.pop() {
                    Some(mesh) => model.add_mesh(
                        nalgebra::Point3::new(0.0, 0.0, 0.0),
                        mesh
                    ),
                    None => break,
                }
            }
        }
    }
}

fn terrain_loaded(renderer: &mut Renderer, data: iohandler::TerrainLoaded) {
    for model in &mut renderer.get_scene().models {
        if model.path == data.path {
            model.add_mesh(
                nalgebra::Point3::new(0.0, 0.0, 0.0),
                data.mesh
            );
            return;
        }
    }
}