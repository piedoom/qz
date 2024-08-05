use std::ops::RangeInclusive;

use bevy::prelude::*;
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Event)]
pub struct SpawnCreature {
    pub name: String,
    pub translation: Vec2,
    pub rotation: f32,
    pub alliegance: Alliegance,
    pub spawner: Option<Entity>,
}

#[derive(Event, Reflect, Serialize, Deserialize, Clone)]
pub struct SpawnGate {
    pub translation: Vec2,
    pub destination: NodeIndex,
}

#[derive(Event, Serialize, Deserialize, Reflect, Clone)]
pub struct SpawnBuilding {
    pub name: String,
    pub translation: Vec2,
    pub rotation: f32,
    pub alliegance: Alliegance,
}

#[derive(Event)]
pub struct GenerateSection {
    /// Define the potential course length of this section in number of zone layers
    pub length: RangeInclusive<usize>,
    /// Define how many potential layers to spawn per Z index.
    pub nodes_per_layer: RangeInclusive<usize>,
}

#[derive(Event)]
pub struct GenerateZone {
    pub node: NodeIndex,
}

/// Despawn all entities associated with the active zone
#[derive(Event)]
pub struct DespawnZone;
