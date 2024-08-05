use bevy::prelude::*;
use petgraph::graph::NodeIndex;

/// References a compliment gate, or none if uninitialized
#[derive(Component, Default, Reflect, Deref, DerefMut, Clone)]
pub struct Gate(NodeIndex);

impl Gate {
    pub fn new(destination: NodeIndex) -> Self {
        Self(destination)
    }
    pub fn destination(&self) -> NodeIndex {
        self.0
    }
}
