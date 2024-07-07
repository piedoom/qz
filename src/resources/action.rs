use bevy::{prelude::*, reflect::Reflect};
use leafwing_input_manager::prelude::*;

/// Actions
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Resource)]
pub enum Action {
    /// Turn left or right
    Turn,
    /// Thrust forwards
    Thrust,
    /// Brake
    Brake,
    /// Fire
    Fire,
}
