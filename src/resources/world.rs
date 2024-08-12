use bevy::prelude::*;
use petgraph::{graph::NodeIndex, prelude::StableGraph, Undirected};

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

/// A node within the [`Universe`]
#[derive(Clone)]
pub struct Zone {
    /// The name identifier of this zone
    pub name: String,
    /// A `Zone` may have an associated scene, if it was already built. If this value is `None`,
    /// a new scene must be generated for this `Zone`.
    pub scene: Option<Handle<ZoneDescription>>,
    /// The depth of this `Zone` in the [`Universe`]. This can help dictate difficulty
    pub depth: usize,
}

impl Zone {
    /// Create a new zone with a depth. This is generally used while generating a new [`Universe` section]
    pub fn new(depth: usize) -> Self {
        Self {
            name: (0..2)
                .map(|_| random_word::gen(random_word::Lang::En).to_string())
                .reduce(|acc, e| acc + " " + &e)
                .unwrap(),
            depth,
            scene: None,
        }
    }
}

/// Current position in the universe
#[derive(Resource, Default)]
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
