use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Thruster control
#[derive(Component, Default, Reflect, Clone, Copy, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Controller {
    /// Linear thrust as a normalized value
    #[serde(skip)]
    pub thrust: f32,
    /// Angular thrust as a normalized value
    #[serde(skip)]
    pub angular_thrust: f32,
    /// Braking force as a normalized value
    #[serde(skip)]
    pub brake: f32,
}
