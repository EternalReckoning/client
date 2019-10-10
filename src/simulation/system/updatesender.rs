use std::sync::mpsc::Sender;

use futures::sync::mpsc::UnboundedSender;
use specs::prelude::*;

use super::super::{
    component::{
        Position,
        ServerID,
    },
    event::{
        Update,
        UpdateEvent,
        PositionUpdate,
        CameraUpdate,
    },
    resource::ActiveCamera,
};

pub struct UpdateSender {
    sender: Sender<Update>,
    net_sender: UnboundedSender<Update>,
}

impl UpdateSender {
    pub fn new(sender: Sender<Update>, net_sender: UnboundedSender<Update>)
        -> UpdateSender
    {
        UpdateSender { sender, net_sender }
    }
}

impl<'a> System<'a> for UpdateSender {
    type SystemData = (
        Entities<'a>,
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, ServerID>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, camera, pos, id) = data;

        for (ent, pos) in (&entities, &pos).join() {
            let time = std::time::Instant::now();
            let event = match Some(ent) == camera.0 {
                true => Update {
                    event: UpdateEvent::CameraUpdate(
                        CameraUpdate(pos.0.clone())
                    ),
                    time,
                },
                false => Update {
                    event: UpdateEvent::PositionUpdate(
                        PositionUpdate {
                            uuid: match id.get(ent) {
                                Some(uuid) => Some(uuid.0),
                                None => None,
                            },
                            position: pos.0.clone(),
                        }
                    ),
                    time,
                },
            };

            self.sender.send(event.clone()).unwrap_or_else(|err| {
                log::error!("failed to send update event: {}", err);
            });

            if id.get(ent).is_none() {
                self.net_sender.unbounded_send(event).unwrap_or_else(|err| {
                    log::error!("failed to send update event: {}", err);
                });
            }
        }
    }
}