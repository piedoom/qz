use bevy::prelude::*;
use petgraph::{Graph, Undirected};

use crate::prelude::*;

/// World generation cursor
#[derive(Resource, Deref, DerefMut, Default)]
pub struct WorldCursor(Slice);

/// Player depth cursor
#[derive(Resource, Deref, DerefMut, Default)]
pub struct DepthCursor(Slice);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct Universe(Graph<Entity, (), Undirected>);
