use specs::prelude::*;

pub struct Health(pub u64);

impl Component for Health {
    type Storage = VecStorage<Self>;
}