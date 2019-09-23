use specs::prelude::*;

pub struct Movement {
    pub speed: f64,
}

impl Component for Movement {
    type Storage = VecStorage<Self>;
}