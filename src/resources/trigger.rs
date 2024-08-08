use std::ops::RangeInclusive;

use bevy::prelude::*;
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
/// Spawn a creature in the world
#[derive(Event)]
pub struct SpawnCreature {
    /// Creature name string
    pub name: String,
    /// Spawn translation
    pub translation: Vec2,
    /// Spawn rotation in radians
    pub rotation: f32,
    /// Alliegance
    pub alliegance: Alliegance,
    /// Spawner, if applicable
    pub spawner: Option<Entity>,
}

/// Spawn a [`Gate`] to another [`Universe`] [`NodeIndex`]
#[derive(Event, Reflect, Serialize, Deserialize, Clone)]
pub struct SpawnGate {
    /// Gate translation
    pub translation: Vec2,
    /// [`Universe`] destination node
    pub destination: NodeIndex,
}

/// Spawn a building
#[derive(Event, Serialize, Deserialize, Reflect, Clone)]
pub struct SpawnBuilding {
    /// Building name string
    pub name: String,
    /// Spawn translation
    pub translation: Vec2,
    /// Spawn rotation
    pub rotation: f32,
    /// Alliegance
    pub alliegance: Alliegance,
}

/// Generate a section of the [`Universe`] from start to end (boss) zones
#[derive(Event)]
pub struct GenerateSection {
    /// Define the potential course length of this section in number of zone layers
    pub length: RangeInclusive<usize>,
    /// Define how many potential layers to spawn per Z index.
    pub nodes_per_layer: RangeInclusive<usize>,
}

/// Generate a new zone (level associated with a [`Universe`] [`NodeIndex`])
#[derive(Event)]
pub struct GenerateZone {
    /// Associated [`NodeIndex`] for this zone
    pub node: NodeIndex,
}

/// Despawn all entities associated with the active zone
#[derive(Event)]
pub struct DespawnZone;
