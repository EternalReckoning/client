use specs::prelude::*;

use eternalreckoning_core::net::operation;

use crate::simulation::{
    event::Event,
    component::{
        Health,
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
        WriteStorage<'a, Health>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, events, mut id, mut hp, mut pos) = data;

        for event in &*events {
            if let Event::NetworkEvent(operation::Operation::SvUpdateWorld(data)) = event {
                for update in &data.updates {
                    let mut entity = None;
                    for (sim_entity, server_id) in (&entities, &id).join() {
                        if server_id.0 == update.uuid {
                            entity = Some(sim_entity);
                            break;
                        }
                    }

                    if entity.is_none() {
                        entity = Some(entities.create());
                        id.insert(entity.unwrap(), ServerID(update.uuid)).unwrap();
                    }
                    let entity = entity.unwrap();

                    for component in &update.data {
                        match component {
                            operation::EntityComponent::Health(data) => {
                                match hp.get_mut(entity) {
                                    Some(ref mut health) => health.0 = *data,
                                    None => {
                                        hp.insert(entity, Health(*data)).unwrap();
                                    },
                                }
                            },
                            operation::EntityComponent::Position(data) => {
                                match pos.get_mut(entity) {
                                    Some(ref mut position) => position.0 = *data,
                                    None => {
                                        pos.insert(entity, Position(*data)).unwrap();
                                    },
                                };
                            },
                        };
                    }
                }
            }
        }
    }
}