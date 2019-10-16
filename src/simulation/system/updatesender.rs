use std::sync::mpsc::Sender;

use futures::sync::mpsc::UnboundedSender;
use specs::prelude::*;

use super::super::{
    component::{
        Model,
        Position,
        ServerID,
        Texture,
    },
    event::{
        Update,
        UpdateEvent,
        PositionUpdate,
        CameraUpdate,
        ModelUpdate,
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

impl<'a> System<'a> for UpdateSender {
    type SystemData = (
        Entities<'a>,
        Read<'a, ActiveCamera>,
        Read<'a, ActiveCharacter>,
        ReadStorage<'a, Model>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, ServerID>,
        ReadStorage<'a, Texture>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, camera, character, model, pos, id, texture) = data;

        let time = std::time::Instant::now();

        // TODO: main loop sender hangup should be fatal

        for (ent, pos) in (&entities, &pos).join() {
            if Some(ent) == camera.0 {
                self.sender.send(Update {
                    event: UpdateEvent::CameraUpdate(
                        CameraUpdate(pos.0.clone())
                    ),
                    time,
                })
                    .unwrap_or_else(|err| {
                        log::error!("failed to send update event: {}", err);
                    });
            }

            let event = Update {
                event: UpdateEvent::PositionUpdate(
                    PositionUpdate {
                        entity: ent,
                        uuid: match id.get(ent) {
                            Some(uuid) => Some(uuid.0),
                            None => None,
                        },
                        position: pos.0.clone(),
                    }
                ),
                time,
            };

            self.sender.send(event.clone()).unwrap_or_else(|err| {
                log::error!("failed to send update event: {}", err);
            });

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
            let event = Update {
                event: UpdateEvent::ModelUpdate(
                    ModelUpdate {
                        entity: ent,
                        path: model.path.clone(),
                        offset: model.offset,
                    }
                ),
                time,
            };

            self.sender.send(event).unwrap_or_else(|err| {
                log::error!("failed to send update event: {}", err);
            });
        }

        for (ent, tex) in (&entities, &texture).join() {
            let event = Update {
                event: UpdateEvent::TextureUpdate(
                    TextureUpdate {
                        entity: ent,
                        path: tex.path.clone(),
                    }
                ),
                time,
            };

            self.sender.send(event).unwrap_or_else(|err| {
                log::error!("failed to send update event: {}", err);
            });
        }
    }
}