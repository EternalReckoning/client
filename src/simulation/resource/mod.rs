mod inputmap;

use super::event::Event;

pub use inputmap::InputMap;

pub type EventQueue = Vec<Event>;