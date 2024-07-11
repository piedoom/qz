use std::time::Duration;

use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Lifetime {
    pub created: Duration,
    pub lifetime: Duration,
}
