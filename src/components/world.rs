use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::DISTANCE_BETWEEN_SLICES;

/// References a compliment gate, or none if uninitialized
#[derive(Component, Default, Reflect, Deref, DerefMut, Clone)]
pub struct Gate(pub Option<Entity>);

impl Gate {
    pub fn new(end_gate: Option<Entity>) -> Self {
        Self(end_gate)
    }
}

#[derive(
    Component,
    Reflect,
    Deref,
    Debug,
    DerefMut,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
)]
pub struct Slice(pub usize);

impl Slice {
    #[inline(always)]
    pub fn z(&self) -> f32 {
        self.0 as f32 * -DISTANCE_BETWEEN_SLICES
    }
}

impl From<usize> for Slice {
    fn from(value: usize) -> Self {
        Slice(value)
    }
}
