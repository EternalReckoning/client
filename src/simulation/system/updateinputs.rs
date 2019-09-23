use specs::prelude::*;

use super::super::{
    event::{
        Event,
        InputEvent,
    },
    resource::{
        EventQueue,
        InputMap,
    },
};

pub struct UpdateInputs;

impl<'a> System<'a> for UpdateInputs {
    type SystemData = (
        Read<'a, EventQueue>,
        Write<'a, InputMap>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (events, mut inputs) = data;

        for event in &*events {
            match event {
                Event::InputEvent(ref event_data) => {
                    match event_data {
                        InputEvent::KeyUp(data) => inputs.set(*data, false),
                        InputEvent::KeyDown(data) => inputs.set(*data, true),
                    };
                },
            }
        }
    }
}