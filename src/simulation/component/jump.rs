use specs::prelude::*;

pub struct Jump {
    pub force: f64,
}

impl Component for Jump {
    type Storage = VecStorage<Self>;
}