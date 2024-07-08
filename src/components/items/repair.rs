use bevy::prelude::*;

#[derive(Clone, Component)]
pub struct RepairBot {
    /// Repair rate / s
    pub rate: f32,
}
