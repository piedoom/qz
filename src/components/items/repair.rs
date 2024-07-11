use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Component, Reflect, Serialize, Deserialize)]
pub struct RepairBot {
    /// Repair rate / s
    pub rate: f32,
}
