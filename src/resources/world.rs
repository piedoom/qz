use bevy::prelude::*;
use petgraph::{graph::NodeIndex, Graph, Undirected};

use crate::prelude::*;

/// Contains a graph of scenes. A section of scenes is generated at a time (from savepoint to boss room(s))
#[derive(Resource, Default)]
pub struct Universe {
    /// Tracks the tail end of the `Universe` where more sections may be appended
    pub end: Vec<NodeIndex>,
    pub graph: Graph<Zone, (), Undirected>,
}

impl Universe {
    pub type GRAPH = Graph<Zone, (), Undirected>;
}

/// A node within the [`Universe`]
pub struct Zone {
    pub name: String,
    /// A `Zone` may have an associated scene, if it was already built. If this value is `None`,
    /// a new scene must be generated for this `Zone`.
    pub scene: Option<Handle<ZoneDescription>>,
    pub depth: usize,
}

impl Zone {
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
    pub fn get(&self) -> NodeIndex {
        self.0
    }
}

impl From<NodeIndex> for UniversePosition {
    fn from(value: NodeIndex) -> Self {
        Self(value)
    }
}
