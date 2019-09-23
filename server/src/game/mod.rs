pub mod component;
pub mod system;
mod game;

pub use game::build_simulation;

pub type EventQueue = Vec<crate::action::ActionEvent>;