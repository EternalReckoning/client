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
    },
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
        ReadStorage<'a, Position>,
        ReadStorage<'a, ServerID>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, pos, id) = data;

        for (ent, pos) in (&entities, &pos).join() {
            let event = Update {
                time: std::time::Instant::now(),
                event: UpdateEvent::PositionUpdate(
                    PositionUpdate {
                        uuid: match id.get(ent) {
                            Some(uuid) => Some(uuid.0),
                            None => None,
                        },
                        position: pos.0.clone(),
                    },
                ),
            };

            self.sender.send(event.clone()).unwrap_or_else(|err| {
                log::error!("failed to send update event: {}", err);
            });

            if id.get(ent).is_none() {
                self.net_sender.send(event).unwrap_or_else(|err| {
                    log::error!("failed to send update event: {}", err);
                });
            }
        }
    }
}