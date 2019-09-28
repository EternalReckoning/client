pub enum Event {
    InputEvent(InputEvent),
}

pub enum InputEvent {
    KeyUp(crate::input::InputTypes),
    KeyDown(crate::input::InputTypes),
    CameraAngle(crate::input::MouseEuler),
}

pub struct Update {
    pub time: std::time::Instant,
    pub event: UpdateEvent,
}

pub enum UpdateEvent {
    PositionUpdate(PositionUpdate),
}

pub struct PositionUpdate {
    pub position: nalgebra::Point3<f64>,
}