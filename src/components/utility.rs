use std::time::Duration;

use bevy::{asset::AssetPath, prelude::*};
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};

/// Despawns an entity after a specified length of time
#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Lifetime {
    /// Timestamp of this entity's birth
    pub created: Duration,
    /// Duration of this entity's life
    pub lifetime: Duration,
}

/// Added to any entity that has a GLTF model and serialized into the scene
#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Model(pub AssetPath<'static>);

#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
pub struct MoveToGate(pub NodeIndex);

/// Added to entities that should persist
#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Persistent;
