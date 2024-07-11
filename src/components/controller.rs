//! Controls thusters

use bevy::prelude::*;

/// Thruster control
#[derive(Component, Default, Reflect)]
pub struct Controller {
    /// Linear thrust as a normalized value
    pub thrust: f32,
    /// Angular thrust as a normalized value
    pub angular_thrust: f32,
    /// Braking force as a normalized value
    pub brake: f32,
}
