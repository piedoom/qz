use bevy::prelude::*;

use crate::prelude::*;

/// World generation cursor
#[derive(Resource, Deref, DerefMut, Default)]
pub struct WorldCursor(Slice);

/// Player depth cursor
#[derive(Resource, Deref, DerefMut, Default)]
pub struct DepthCursor(Slice);
