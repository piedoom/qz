use std::ops::RangeInclusive;

use bevy::{prelude::*, utils::HashSet};
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

/// Generate a new chunk
#[derive(Event)]
pub struct GenerateChunks {
    /// Generate at this chunk position
    pub chunk_indicies: Vec<ChunkIndex>,
}
