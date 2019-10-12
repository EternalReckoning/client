use specs::prelude::*;

pub struct Movement {
    pub speed: f64,
    pub on_ground: bool,
}

impl Component for Movement {
    type Storage = VecStorage<Self>;
}