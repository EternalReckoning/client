use uuid::Uuid;

pub enum Event {
    InputEvent(InputEvent),
    NetworkEvent(NetworkEvent),
}

#[derive(Debug)]
pub enum InputEvent {
    KeyUp(crate::input::InputTypes),
    KeyDown(crate::input::InputTypes),
    CameraAngle(crate::input::MouseEuler),
}

pub enum NetworkEvent {
    WorldUpdate(WorldUpdate),
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
    pub uuid: Option<Uuid>,
    pub position: nalgebra::Point3<f64>,
}

pub struct WorldUpdate {
    pub updates: Vec<EntityUpdate>,
}

pub struct EntityUpdate {
    pub uuid: Uuid,
    pub position: nalgebra::Point3::<f64>,
}