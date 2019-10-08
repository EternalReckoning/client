use std::time::{ Duration, Instant };
use std::thread;
use std::sync::mpsc::{
    channel,
    Sender,
    Receiver,
};

use failure::{
    format_err,
    Error,
};
use futures::sync::mpsc::unbounded;
use rendy::{
    factory::Factory,
    wsi::winit,
};

use crate::{
    input,
    input::InputTypes,
    networking,
    renderer,
    simulation::{
        build_simulation,
        event,
    },
    util::config,
    window::Window,
};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ClientConfig {
    pub server_address: String,
}

impl Default for ClientConfig {
    fn default() -> ClientConfig {
        ClientConfig {
            server_address: "127.0.0.1:6142".to_string(),
        }
    }
}

type Backend = rendy::vulkan::Backend;

const TICK_RATE: u64 = 60;

fn run(
    window: Window,
    mut factory: Factory<Backend>,
    mut families: rendy::command::Families<Backend>,
    mut scene: renderer::scene::Scene,
    graph: renderer::RenderGraph<Backend>,
    config: config::Config,
    event_tx: Sender<event::Event>,
    update_rx: Receiver<event::Update>,
) -> Result<(), Error> {
    let started = std::time::Instant::now();

    let mut key_map = std::collections::HashMap::<u32, InputTypes>::new();
    key_map.insert(config.key_map.move_forward, InputTypes::MoveForward);
    key_map.insert(config.key_map.move_left, InputTypes::MoveLeft);
    key_map.insert(config.key_map.move_backward, InputTypes::MoveBackward);
    key_map.insert(config.key_map.move_right, InputTypes::MoveRight);
    key_map.insert(config.key_map.move_up, InputTypes::MoveUp);

    let mut frame = 0u64;
    let mut period = started;
    let mut graph = Some(graph);

    let mouse_sens = input::MouseSensitivity::new(config.mouse.sensitivity);
    let mut mouse_euler = input::MouseEuler::default();
    let mut camera_pos = nalgebra::Point3::<f64>::new(0.0, 0.0, 0.0);
    let mut mouse_look = false;

    window.run(move |event, _, control_flow| {
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
            winit::event::Event::EventsCleared => {
                loop {
                    let update = update_rx.try_recv();
                    match update {
                        Ok(e) => {
                            let event::UpdateEvent::PositionUpdate(event::PositionUpdate { uuid, position }) = e.event;
                            match uuid {
                                Some(uuid) => {
                                    let position = nalgebra::Transform3::<f32>::identity() * 
                                        nalgebra::Translation3::new(
                                            position.x as f32,
                                            position.y as f32,
                                            position.z as f32
                                        );
                                    let mut found = false;
                                    for object in &mut scene.objects {
                                        if object.id.is_some() && object.id.unwrap() == uuid {
                                            found = true;
                                            object.position = position;
                                            break;
                                        }
                                    }
                                    if !found {
                                        scene.objects.push(renderer::scene::Object {
                                            id: Some(uuid),
                                            model: 1,
                                            position
                                        });
                                    }
                                },
                                None => camera_pos = position,
                            };
                        },
                        Err(_) => break,
                    }
                }

                let rotation = nalgebra::Rotation3::from_euler_angles(
                    mouse_euler.pitch as f32,
                    mouse_euler.yaw as f32,
                    0.0,
                );
                let position = nalgebra::Translation3::new(
                    camera_pos.x as f32,
                    camera_pos.y as f32,
                    camera_pos.z as f32
                );
                let translation = nalgebra::Translation3::<f32>::new(0.0, 0.0, 10.0);
                scene.camera.set_view(
                    nalgebra::Projective3::identity() * position * rotation * translation
                );

                scene.objects[1].position = nalgebra::convert(position);

                factory.maintain(&mut families);

                if let Some(ref mut graph) = graph {
                    graph.run(&mut factory, &mut families, &scene);
                    frame += 1;
                }

                if period.elapsed() >= Duration::new(5, 0) {
                    period = Instant::now();
                    let elapsed = started.elapsed();
                    let elapsed_ns = elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64;

                    log::info!(
                        "Elapsed: {:?}. Frames: {}. FPS: {}",
                        elapsed,
                        frame,
                        frame * 1_000_000_000 / elapsed_ns
                    );
                }
            },
            _ => {},
        }

        // TODO: do io outside the rendering thread
        for model in &mut scene.models {
            if model.len() == 0 {
                model.load().unwrap_or_else(|err| log::error!("Error: {}", err));
            }
        }

        if *control_flow == winit::event_loop::ControlFlow::Exit && graph.is_some() {
            log::info!("Exiting...");
            graph.take().unwrap().dispose(&mut factory, &scene);
        }
    });

    Ok(())
}

pub fn main(config: config::Config) -> Result<(), Error> {
    let rendy_config: rendy::factory::Config = Default::default();
    let (mut factory, mut families): (Factory<Backend>, _) =
        rendy::factory::init(rendy_config)
            .map_err(|err| format_err!("failed to configure graphics device: {:?}", err))?;

    log::info!("Creating window...");

    let window = Window::new()?;

    log::info!("Initializing rendering pipeline...");

    let aspect = window.get_aspect_ratio() as f32;

    let mut scene = renderer::scene::Scene {
        camera: renderer::scene::Camera::new(aspect),
        ui: renderer::scene::UI::new(aspect),
        models: vec![
            renderer::Model::new("assets/floor.erm".to_string()),
            renderer::Model::new("assets/marker.erm".to_string()),
            renderer::Model::new("assets/pillar.erm".to_string()),
            renderer::Model::new("assets/elf-spear.erm".to_string()),
        ],
        objects: vec![
            renderer::scene::Object {
                id: None,
                model: 0,
                position: nalgebra::Transform3::identity() *
                    nalgebra::Translation3::new(0.0, 0.0, 0.0),
            },
            renderer::scene::Object {
                id: None,
                model: 1,
                position: nalgebra::Transform3::identity() *
                    nalgebra::Translation3::new(0.0, 0.0, 0.0),
            },
            renderer::scene::Object {
                id: None,
                model: 2,
                position: nalgebra::Transform3::identity() *
                    nalgebra::Translation3::new(-5.5, 0.0, -7.0),
            },
            renderer::scene::Object {
                id: None,
                model: 2,
                position: nalgebra::Transform3::identity() *
                    nalgebra::Translation3::new(5.5, 0.0, -7.0),
            },
            renderer::scene::Object {
                id: None,
                model: 3,
                position: nalgebra::Transform3::identity() *
                    nalgebra::Translation3::new(0.0, 0.0, -9.0),
            },
        ],
    };

    let graph = renderer::RenderGraph::new(
        &mut factory,
        &mut families,
        &mut scene,
        &window,
    );

    let (event_tx, event_rx) = channel();
    let (net_update_tx, net_update_rx) = unbounded();

    log::info!("Initializing networking");
    
    let net_event_tx = event_tx.clone();
    let addr = config.client.server_address.clone();
    thread::spawn(move || {
        networking::connect(
            &addr,
            net_update_rx,
            net_event_tx
        );
    });
    
    log::info!("Initializing simulation");
    
    let tick_length = Duration::from_millis(
        1000 / TICK_RATE
    );

    let (main_update_tx, main_update_rx) = channel();
    let sim_config = config.simulation.clone();
    thread::spawn(move || {
        let mut game = build_simulation(
            sim_config,
            main_update_tx,
            net_update_tx,
            tick_length
        );
        game.run(event_rx, tick_length);
    });

    log::info!("Entering main loop");
    
    run(window, factory, families, scene, graph, config, event_tx, main_update_rx)
}