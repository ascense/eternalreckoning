pub mod component;
pub mod system;
mod simulation;
mod event;

pub use event::Event;
pub use simulation::build_simulation;

pub type EventQueue = Vec<Event>;