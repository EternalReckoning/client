mod inputmap;
mod ticklength;

use super::event::Event;

pub use inputmap::InputMap;
pub use ticklength::TickLength;

pub type EventQueue = Vec<Event>;