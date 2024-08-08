use std::time::Duration;

use bevy::prelude::*;

/// Despawns an entity after a specified length of time
#[derive(Component, Reflect)]
pub struct Lifetime {
    /// Timestamp of this entity's birth
    pub created: Duration,
    /// Duration of this entity's life
    pub lifetime: Duration,
}
