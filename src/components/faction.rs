//! Non players

use bevy::prelude::*;

/// Tracks things in a specified range
#[derive(Component)]
pub struct InRange {
    pub range: f32,
    pub allies: Vec<Entity>,
    pub enemies: Vec<Entity>,
}

impl InRange {
    pub fn new(range: f32) -> Self {
        Self { range, ..default() }
    }
    pub fn clear(&mut self) {
        self.allies.clear();
        self.enemies.clear();
    }
}

impl Default for InRange {
    fn default() -> Self {
        Self {
            range: 1f32,
            allies: default(),
            enemies: default(),
        }
    }
}

#[derive(Component, Copy, Clone)]
pub struct Alliegance {
    /// The faction of the entity
    pub faction: Faction,
    /// Allied factions of this entity
    pub allies: Faction,
    /// Enemy factions of this entity
    pub enemies: Faction,
}

bitflags::bitflags! {
    #[derive(PartialEq, Eq, Copy, Clone)]
    pub struct Faction: u32 {
        const PLAYER = 1;
        const ENEMY = 2;
    }
}
