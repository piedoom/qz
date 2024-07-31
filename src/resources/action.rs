use bevy::{prelude::*, reflect::Reflect};
use leafwing_input_manager::prelude::*;

/// Actions
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Resource)]
pub enum Action {
    /// Turn left or right
    Turn,
    /// Thrust forwards and brake when negative
    Thrust,
    /// Fire
    Fire,
    /// Take
    Take,
    /// Interact
    Interact,
}

impl Actionlike for Action {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Action::Turn | Action::Thrust => InputControlKind::Axis,
            Action::Fire | Action::Interact | Action::Take => InputControlKind::Button,
        }
    }
}

/// Application level actions that shouldn't be tied to any specific entity
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Resource)]
pub enum AppAction {
    /// Show debug tools
    Console,
}
