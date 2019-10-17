use uuid::Uuid;
use eternalreckoning_core::net::operation::Operation;

pub enum Event {
    ConnectionEvent(ConnectionEvent),
    InputEvent(InputEvent),
    NetworkEvent(Operation),
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Connected(Uuid),
    Disconnected(Uuid),
}

#[derive(Debug)]
pub enum InputEvent {
    KeyUp(crate::input::InputTypes),
    KeyDown(crate::input::InputTypes),
    CameraAngle(crate::input::MouseEuler),
}

#[derive(Clone)]
pub enum Update {
    SimulationTick(std::time::Instant),
    CameraUpdate(CameraUpdate),
    ModelUpdate(ModelUpdate),
    PositionUpdate(PositionUpdate),
    TextureUpdate(TextureUpdate),
}

#[derive(Clone)]
pub struct CameraUpdate(pub nalgebra::Point3<f64>);

#[derive(Clone)]
pub struct ModelUpdate {
    pub entity: specs::Entity,
    pub path: String,
    pub offset: Option<nalgebra::Vector3<f32>>,
}

#[derive(Clone)]
pub struct TextureUpdate {
    pub entity: specs::Entity,
    pub path: String,
}

#[derive(Clone)]
pub struct PositionUpdate {
    pub entity: specs::Entity,
    pub uuid: Option<Uuid>,
    pub position: nalgebra::Point3<f64>,
}