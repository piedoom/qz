use bevy::prelude::*;

/// Determines whether to draw debug UI
#[derive(Default, Resource, Deref, DerefMut, PartialEq, Eq)]
pub struct DrawInspector(pub bool);

/// Name of the saved game
#[derive(Default, Resource, Deref, DerefMut, Clone)]
pub struct SaveGameName(pub String);
