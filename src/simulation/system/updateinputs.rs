use specs::prelude::*;

use crate::input::MouseEuler;
use crate::simulation::{
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
        Write<'a, MouseEuler>,
        Write<'a, InputMap>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (events, mut mouse_euler, mut inputs) = data;

        for event in &*events {
            match event {
                Event::InputEvent(ref event_data) => {
                    match event_data {
                        InputEvent::KeyUp(data) => inputs.set(*data, false),
                        InputEvent::KeyDown(data) => inputs.set(*data, true),
                        InputEvent::CameraAngle(data) => {
                            mouse_euler.pitch = data.pitch;
                            mouse_euler.yaw = data.yaw;
                        },
                    };
                },
                _ => (),
            }
        }
    }
}