//! Fixtures in space

use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};

use crate::prelude::*;

#[derive(Component, Reflect)]
pub struct Structure;

/// Added when a craft is docked, and removed when undocked
#[derive(Component, Reflect, Deref, DerefMut)]
pub struct Docked(pub Entity);

/// Updated to determine the nearest dockable structure
#[derive(Component, Reflect)]
pub struct DockInRange {
    pub dock: Option<Entity>,
    pub range: f32,
}

#[derive(Component, Reflect)]
pub struct Spawner {
    pub maximum: usize,
    pub delay: Duration,
    pub last_spawned: Duration,
}

/// Used to track the maximum created from our spawner
#[derive(Component, Reflect)]
pub struct SpawnedFrom(pub Entity);

/// Can dock at this. Maps a docked entity to its constraint entity
#[derive(Component, Reflect, Default, Deref, DerefMut)]
pub struct Dockings(pub HashMap<Entity, Entity>);

#[derive(Component, Default)]
pub struct Store {
    pub items: HashMap<Handle<Item>, SaleOptions>,
}

/// Describes whether an item is listed for sale or for buying, as well as the
/// deviation from base price to list
pub struct SaleOptions {
    pub sell: Option<usize>,
    pub buy: Option<usize>,
}

impl SaleOptions {}

impl Default for SaleOptions {
    fn default() -> Self {
        Self {
            sell: None,
            buy: None,
        }
    }
}
