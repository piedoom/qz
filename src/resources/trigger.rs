use bevy::prelude::*;

use crate::prelude::*;

#[derive(Event)]
pub struct SpawnCreature {
    pub name: String,
    pub slice: Slice,
    pub translation: Vec2,
    pub rotation: f32,
    pub alliegance: Alliegance,
    pub spawner: Option<Entity>,
}

#[derive(Event)]
pub struct SpawnGate {
    /// If specified, will use the provided entity to spawn the gate onto
    pub use_entity: Option<Entity>,
    pub slice: Slice,
    pub translation: Vec2,
    pub end_gate: Option<Entity>,
}

#[derive(Event)]
pub struct SpawnBuilding {
    pub name: String,
    pub slice: Slice,
    pub translation: Vec2,
    pub rotation: f32,
    pub alliegance: Alliegance,
}

#[derive(Event)]
pub struct SpawnSlice {
    pub from_gate: Option<Entity>,
    pub slice: Slice,
}

#[derive(Event)]
pub struct Save;

#[derive(Event)]
pub struct Load;
