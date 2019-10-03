use specs::prelude::*;

use crate::simulation::{
    event::{
        Event,
        NetworkEvent,
    },
    component::{
        Position,
        ServerID,
    },
    resource::EventQueue,
};

pub struct UpdateWorld;

impl<'a> System<'a> for UpdateWorld {
    type SystemData = (
        Entities<'a>,
        Read<'a, EventQueue>,
        WriteStorage<'a, ServerID>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, events, mut id, mut pos) = data;

        for event in &*events {
            if let Event::NetworkEvent(NetworkEvent::WorldUpdate(data)) = event {
                for entity in &data.updates {
                    let mut found = false;
                    for (server_id, position) in (&id, &mut pos).join() {
                        if server_id.0 == entity.uuid {
                            found = true;
                            position.0 = entity.position;
                        }
                    }
                    if !found {
                        let new_entity = entities.create();
                        id.insert(new_entity, ServerID(entity.uuid));
                        pos.insert(new_entity, Position(entity.position));
                    }
                }
            }
        }
    }
}