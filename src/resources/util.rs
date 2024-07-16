use bevy::prelude::*;

/// Determines whether to draw debug UI
#[derive(Default, Resource, Deref, DerefMut, PartialEq, Eq)]
pub struct DrawInspector(pub bool);
