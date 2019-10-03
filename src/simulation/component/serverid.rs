use specs::prelude::*;
use uuid::Uuid;

pub struct ServerID(pub Uuid);

impl Component for ServerID {
    type Storage = VecStorage<Self>;
}