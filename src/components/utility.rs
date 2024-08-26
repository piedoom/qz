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

/// Despawns an entity after a specified distance of time
#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct DistanceLifetime {
    /// Location of this entity's birth
    pub created: Vec3,
    /// Length that this entity can travel before despawning
    pub length: f32,
}

/// Added to any entity that has a GLTF model and serialized into the scene
#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Model {
    path: AssetPath<'static>,
    offset: Vec3,
}

impl Model {
    pub fn new(path: impl Into<Handle<Scene>>) -> Self {
        Self {
            path: path.into().path().unwrap().clone(),
            offset: Vec3::ZERO,
        }
    }
    pub fn path(&self) -> &AssetPath {
        &self.path
    }
    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    pub(crate) fn offset(&self) -> Vec3 {
        self.offset
    }
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
pub struct MoveToGate(pub NodeIndex);

/// Added to entities that should persist
#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Persistent;
