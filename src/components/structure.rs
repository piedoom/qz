//! Fixtures in space

use std::time::Duration;

use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Structure;

#[derive(Component, Reflect)]
pub struct Spawner {
    pub maximum: usize,
    pub delay: Duration,
    pub last_spawned: Duration,
}

/// Used to track the maximum created from our spawner
#[derive(Component, Reflect)]
pub struct SpawnedFrom(pub Entity);
