use uuid::Uuid;
use eternalreckoning_core::net::operation::Operation;

pub enum Event {
    InputEvent(InputEvent),
    NetworkEvent(Operation),
}

#[derive(Debug)]
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
    CameraUpdate(CameraUpdate),
}

#[derive(Clone)]
pub struct PositionUpdate {
    pub uuid: Option<Uuid>,
    pub position: nalgebra::Point3<f64>,
}

#[derive(Clone)]
pub struct CameraUpdate(pub nalgebra::Point3<f64>);