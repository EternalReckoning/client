use specs::prelude::*;

use eternalreckoning_core::net::operation;

use crate::simulation::{
    event::{
        Event,
        ConnectionEvent,
    },
    component::{
        Model,
        Texture,
        Health,
        Position,
        ServerID,
    },
    resource::{
        ActiveCharacter,
        EventQueue,
    },
};

pub struct UpdateWorld;

impl<'a> System<'a> for UpdateWorld {
    type SystemData = (
        Entities<'a>,
        Read<'a, EventQueue>,
        Read<'a, ActiveCharacter>,
        WriteStorage<'a, ServerID>,
        WriteStorage<'a, Model>,
        WriteStorage<'a, Texture>,
        WriteStorage<'a, Health>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, events, character, mut id, mut model, mut texture, mut hp, mut pos) = data;

        for event in &*events {
            match event {
                Event::ConnectionEvent(ConnectionEvent::Connected(uuid)) => {
                    if let Some(entity) = character.0 {
                        match id.get_mut(entity) {
                            Some(ref mut id) => id.0 = uuid.clone(),
                            None => {
                                id.insert(entity, ServerID(uuid.clone()))
                                    .unwrap_or_else(|_| {
                                        log::warn!("failed to set player UUID");
                                        None
                                    });
                            },
                        };
                    }
                },
                Event::ConnectionEvent(ConnectionEvent::Disconnected(_)) => {
                    if let Some(entity) = character.0 {
                        id.remove(entity);
                    };
                },
                Event::NetworkEvent(op) => {
                    match op {
                        operation::Operation::SvUpdateWorld(data) => {
                            for update in &data.updates {
                                let mut entity = None;
                                for (sim_entity, server_id) in (&entities, &id).join() {
                                    if server_id.0 == update.uuid {
                                        entity = Some(sim_entity);
                                        break;
                                    }
                                }

                                if entity.is_none() {
                                    log::debug!("New entity: {}", update.uuid);
                                    entity = Some(entities.create());
                                    id.insert(entity.unwrap(), ServerID(update.uuid)).unwrap();
                                    model.insert(entity.unwrap(), Model::new("assets/marker.erm")).unwrap();
                                    texture.insert(entity.unwrap(), Texture::new("assets/marker.png")).unwrap();
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
                        },
                        _ => (),
                    };
                },
                Event::InputEvent(_) => (),
            }
        }
    }
}