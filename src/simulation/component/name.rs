use specs::prelude::*;

pub struct Name(pub String);

impl Component for Name {
    type Storage = VecStorage<Self>;
}