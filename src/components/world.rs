use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::DISTANCE_BETWEEN_SLICES;

#[derive(Component, Reflect, Deref, DerefMut, Clone)]
pub struct Gate(pub Slice);

impl Gate {
    pub fn new(target_slice: impl Into<Slice>) -> Self {
        Self(target_slice.into())
    }
}

#[derive(
    Component, Reflect, Deref, Debug, DerefMut, Default, Clone, Copy, Serialize, Deserialize,
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
