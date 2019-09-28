pub enum Event {
    InputEvent(InputEvent),
    NetworkEvent,
}

pub enum InputEvent {
    KeyUp(crate::input::InputTypes),
    KeyDown(crate::input::InputTypes),
    CameraAngle(crate::input::MouseEuler),
}

#[derive(Clone)]
pub struct Update {
    pub time: std::time::Instant,
    pub event: UpdateEvent,
}

#[derive(Clone)]
pub enum UpdateEvent {
    PositionUpdate(PositionUpdate),
}

#[derive(Clone)]
pub struct PositionUpdate {
    pub position: nalgebra::Point3<f64>,
}