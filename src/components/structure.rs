//! Fixtures in space

use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// A permanent, usually non-moving thing in the game such as a station or a gate
#[derive(Component, Reflect)]
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
#[derive(Component, Reflect)]
pub struct SpawnedFrom(pub Entity);

/// Can dock at this. Maps a docked entity to its constraint entity
#[derive(Component, Reflect, Default, Deref, DerefMut)]
pub struct Dockings(pub HashMap<Entity, Entity>);

/// Marks a building as a store that can be traded with, if docked
#[derive(Component, Default, Reflect)]
pub struct Store {
    /// Items that are bought and sold at this station
    pub items: HashMap<Handle<Item>, SaleOptions>,
}

/// Describes whether an item is listed for sale or for buying
/// Options for fine-tuning sales
#[derive(Default, Reflect, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SaleOptions {
    /// Amount to sell at
    pub sell: SaleOption,
    /// Amount to buy at
    pub buy: SaleOption,
}

impl SaleOptions {}
/// Options for buying and selling
#[derive(Default, Reflect, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SaleOption {
    /// Not for sale/purchase
    #[default]
    None,
    /// Scale the base price determined on the item level
    Scaled(f32),
    /// Set a specific price
    Absolute(usize),
}

impl SaleOption {
    /// Returns `false` if `None`
    #[inline(always)]
    pub fn enabled(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Get the adjusted value of this item given its base value
    pub fn value(&self, base_value: usize) -> Option<usize> {
        match self {
            SaleOption::None => None,
            SaleOption::Scaled(scalar) => Some((base_value as f32 * scalar) as usize),
            SaleOption::Absolute(value) => Some(*value),
        }
    }
}
