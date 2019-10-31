use std::sync::mpsc::Sender;

use futures::sync::mpsc::UnboundedSender;
use specs::prelude::*;

use eternalreckoning_core::simulation::TickTime;

use super::super::{
    component::{
        Model,
        Position,
        ServerID,
        Terrain,
        Texture,
    },
    event::{
        Update,
        PositionUpdate,
        CameraUpdate,
        ModelUpdate,
        TerrainUpdate,
        TextureUpdate,
    },
    resource::{
        ActiveCamera,
        ActiveCharacter,
    },
};

pub struct UpdateSender {
    sender: Sender<Update>,
    net_sender: Option<UnboundedSender<Update>>,
}

impl UpdateSender {
    pub fn new(sender: Sender<Update>, net_sender: UnboundedSender<Update>)
        -> UpdateSender
    {
        UpdateSender { sender, net_sender: Some(net_sender) }
    }
}

impl UpdateSender {
    fn send_event(&self, event: Update) {
        self.sender.send(event)
            .unwrap_or_else(|err| {
                log::error!("failed to send update event: {}", err);
            });
    }
}

impl<'a> System<'a> for UpdateSender {
    type SystemData = (
        Entities<'a>,
        Read<'a, TickTime>,
        Read<'a, ActiveCamera>,
        Read<'a, ActiveCharacter>,
        ReadStorage<'a, Model>,
        ReadStorage<'a, Terrain>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, ServerID>,
        ReadStorage<'a, Texture>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            tick_time,
            camera,
            character,
            model,
            terrain,
            pos,
            id,
            texture
        ) = data;

        // TODO: main loop sender hangup should be fatal

        self.send_event(Update::SimulationTick(tick_time.0));

        for (ent, pos) in (&entities, &pos).join() {
            if Some(ent) == camera.0 {
                self.send_event(Update::CameraUpdate(
                    CameraUpdate(pos.0.clone())
                ));
            }

            let event = Update::PositionUpdate(
                PositionUpdate {
                    entity: ent,
                    uuid: match id.get(ent) {
                        Some(uuid) => Some(uuid.0),
                        None => None,
                    },
                    position: pos.0.clone(),
                }
            );

            self.send_event(event.clone());

            if let Some(net_sender) = &self.net_sender {
                if character.0.is_some() && ent == character.0.unwrap() {
                    net_sender.unbounded_send(event).unwrap_or_else(|err| {
                        log::error!("failed to send update event: {}", err);
                        self.net_sender = None;
                    });
                }
            }
        }

        for (ent, model) in (&entities, &model).join() {
            self.send_event(Update::ModelUpdate(
                ModelUpdate {
                    entity: ent,
                    path: model.path.clone(),
                    offset: model.offset,
                }
            ));
        }

        for (ent, terrain) in (&entities, &terrain).join() {
            self.send_event(Update::TerrainUpdate(
                TerrainUpdate {
                    entity: ent,
                    heightmap: terrain.path.clone(),
                    scale: terrain.scale,
                }
            ));
        }

        for (ent, tex) in (&entities, &texture).join() {
            self.send_event(Update::TextureUpdate(
                TextureUpdate {
                    entity: ent,
                    path: tex.path.clone(),
                    wrap_mode: tex.wrap_mode,
                }
            ));
        }
    }
}