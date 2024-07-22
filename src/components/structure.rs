//! Fixtures in space

use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Component, Reflect, Serialize, Deserialize)]
pub struct Spawner {
    pub spawns: Vec<(String, usize)>,
    pub maximum: usize,
    pub tick: f32,
    #[serde(skip)]
    pub last_tick: Duration,
}

/// Used to track the maximum created from our spawner
#[derive(Component, Reflect)]
pub struct SpawnedFrom(pub Entity);

/// Can dock at this. Maps a docked entity to its constraint entity
#[derive(Component, Reflect, Default, Deref, DerefMut)]
pub struct Dockings(pub HashMap<Entity, Entity>);

#[derive(Component, Default, Reflect)]
pub struct Store {
    pub items: HashMap<Handle<Item>, SaleOptions>,
}

/// Describes whether an item is listed for sale or for buying
#[derive(Default, Reflect, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SaleOptions {
    pub sell: SaleOption,
    pub buy: SaleOption,
}

impl SaleOptions {}

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
    #[inline(always)]
    pub fn enabled(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn value(&self, base_value: usize) -> Option<usize> {
        match self {
            SaleOption::None => None,
            SaleOption::Scaled(scalar) => Some((base_value as f32 * scalar) as usize),
            SaleOption::Absolute(value) => Some(*value),
        }
    }
}
