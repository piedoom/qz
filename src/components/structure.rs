//! Fixtures in space

use std::time::Duration;

use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// A permanent, usually non-moving thing in the game such as a station or a gate
#[derive(Component, Reflect, Serialize, Deserialize, Debug, Clone, Copy)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Structure;

/// Added when a craft is docked, and removed when undocked
#[derive(Component, Reflect, Deref, DerefMut)]
pub struct Docked(pub Entity);

/// Updated to determine the nearest dockable structure
/// Stores a dockable entity in range
#[derive(Component, Reflect)]
pub struct DockInRange {
    /// The dock in range, if it exists
    pub dock: Option<Entity>,
    /// The max search radius from the entity
    pub range: f32,
}

/// Building that spawns creatures
#[derive(Clone, Debug, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Spawner {
    /// Names of the creature asset that will spawn, paired with the `d` likliehood it will be spawned that tick
    pub spawns: Vec<(String, usize)>,
    /// Maximum number of spawns that can be spawned by this spawner at once
    pub maximum: usize,
    /// Duration between ticks. One creature has the chance to spawn per tick.
    pub tick: f32,
    /// The timestamp of the last tick
    #[serde(skip)]
    pub last_tick: Duration,
}

/// Used to track the maximum created from our spawner
#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct SpawnedFrom(pub Entity);

/// Can dock at this. Maps a docked entity to its constraint entity
#[derive(Component, Reflect, Default, Deref, DerefMut, Serialize, Deserialize, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Dockings(#[reflect(skip_serializing)] pub HashMap<Entity, Entity>);

/// Marks a building as a store that can be traded with, if docked
#[derive(Component, Default, Reflect)]
pub struct Store {
    /// Items for sale
    pub items: HashSet<Handle<Item>>,
    /// Difference between buy and sell price, as a percentage
    pub margin: f32,
}
