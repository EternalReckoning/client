use specs::prelude::*;

pub struct Collider {
    pub collider: ColliderType,
    pub collisions: Vec<Collision>,
}

pub struct Collision {
    pub with: Entity,
    pub depth: nalgebra::Vector3<f64>,
}

pub enum ColliderType {
    Plane(nalgebra::Unit<nalgebra::Vector3<f64>>),
    Sphere(f64)
}

impl Collider {
    pub fn new(collider: ColliderType) -> Collider {
        Collider {
            collider,
            collisions: Vec::new(),
        }
    }
}

impl Component for Collider {
    type Storage = VecStorage<Self>;
}