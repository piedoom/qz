use bevy::prelude::*;

#[derive(Component, Reflect, Deref, DerefMut, Clone)]
pub struct Gate(pub Slice);

impl Gate {
    pub fn new(target_slice: impl Into<Slice>) -> Self {
        Self(target_slice.into())
    }
}

#[derive(Component, Reflect, Deref, DerefMut, Default, Clone)]
pub struct Slice(pub usize);
