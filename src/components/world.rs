use bevy::prelude::*;
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};

/// References a compliment gate, or none if uninitialized
#[derive(Component, Default, Reflect, Deref, DerefMut, Clone, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Gate(NodeIndex);

impl Gate {
    /// Creates a new gate with a destination leading to a node in our `Universe` graph
    pub fn new(destination: NodeIndex) -> Self {
        Self(destination)
    }
    /// The destination of this gate, represented as the node in our `Universe` graph
    pub fn destination(&self) -> NodeIndex {
        self.0
    }
}
