use bevy::prelude::*;
use petgraph::{graph::NodeIndex, prelude::StableGraph, Undirected};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// The underlying type of our universe and its connecting zones
pub type UniverseGraph = StableGraph<Zone, (), Undirected>;

/// Contains a graph of scenes. A section of scenes is generated at a time (from savepoint to boss room(s))
#[derive(Resource, Default)]
pub struct Universe {
    /// Tracks the tail end of the `Universe` where more sections may be appended
    pub end: Vec<NodeIndex>,
    /// The graph of the levels in this `Universe`
    pub graph: UniverseGraph,
}

#[derive(Serialize, Deserialize, Default)]
pub struct UniverseSerialized {
    /// Tracks the tail end of the `Universe` where more sections may be appended
    pub end: Vec<NodeIndex>,
    /// The graph of the levels in this `Universe`
    pub graph: StableGraph<ZoneSerialized, (), Undirected>,
}

/// Current position in the universe
#[derive(Resource, Default, Serialize, Deserialize, Clone, Copy)]
pub struct UniversePosition(pub NodeIndex);

impl UniversePosition {
    /// Get the current position of the universe
    pub fn get(&self) -> NodeIndex {
        self.0
    }
}

impl From<NodeIndex> for UniversePosition {
    fn from(value: NodeIndex) -> Self {
        Self(value)
    }
}
