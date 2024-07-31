//! Non players

pub mod actions;
pub mod scorers;

use bevy::prelude::*;

/// Tracks things in a specified range
#[derive(Component, Reflect)]
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

#[derive(Component)]
pub enum Waypoint {
    Entity(Entity),
    Position(Vec2),
}

#[derive(Component)]
pub enum Targeting {
    Entity(Entity),
    Position(Vec2),
}
