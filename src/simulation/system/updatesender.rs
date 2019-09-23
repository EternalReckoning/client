use std::sync::mpsc::Sender;

use specs::prelude::*;

use super::super::{
    component::{
        Position,
    },
    event::{
        Update,
        UpdateEvent,
        PositionUpdate,
    },
};

pub struct UpdateSender {
    sender: Sender<Update>,
}

impl UpdateSender {
    pub fn new(sender: Sender<Update>) -> UpdateSender {
        UpdateSender { sender }
    }
}

impl<'a> System<'a> for UpdateSender {
    type SystemData = ReadStorage<'a, Position>;

    fn run(&mut self, pos: Self::SystemData) {
        for pos in pos.join() {
            self.sender.send(
                Update {
                    time: std::time::Instant::now(),
                    event: UpdateEvent::PositionUpdate(
                        PositionUpdate {
                            position: pos.0.clone(),
                        },
                    ),
                }
            ).unwrap();
        }
    }
}