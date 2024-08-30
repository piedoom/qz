use std::path::PathBuf;

use bevy::{prelude::*, utils::HashSet};
use serde::{Deserialize, Serialize};
/// Determines whether to draw debug UI
#[derive(Default, Resource, Deref, DerefMut, PartialEq, Eq)]
pub struct DrawInspector(pub bool);

#[derive(Resource, Default)]
pub struct Chunks {
    /// Chunks that are already loaded and spawned into the game
    loaded: HashSet<ChunkIndex>,
}

impl Chunks {
    pub fn is_generated(&self, chunk_index: &ChunkIndex) -> bool {
        self.loaded.contains(chunk_index)
    }
    pub fn insert(&mut self, k: ChunkIndex) {
        self.loaded.insert(k);
    }
}

/// Units in a chunk
const CHUNK_SIZE: usize = 128;
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct ChunkIndex(IVec2);

impl ChunkIndex {
    /// Get the offset of this chunk in world units
    pub fn to_world_coordinates(&self) -> Vec2 {
        let coord = self.0 * CHUNK_SIZE as i32;
        Vec2::new(coord.x as f32, coord.y as f32)
    }
    pub fn from_world_coordinates(value: Vec2) -> Self {
        let coord = (value / Vec2::splat(CHUNK_SIZE as f32)).round();
        Self(IVec2::new(coord.x as i32, coord.y as i32))
    }
}

/// The currently loaded/saved world (We'll also deserialize this from our save)
#[derive(Resource, Deref, DerefMut, Default, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct SavePath(pub Option<PathBuf>);

impl From<IVec2> for ChunkIndex {
    fn from(value: IVec2) -> Self {
        Self(value)
    }
}

impl From<(i32, i32)> for ChunkIndex {
    fn from(value: (i32, i32)) -> Self {
        Self(IVec2::new(value.0, value.1))
    }
}
