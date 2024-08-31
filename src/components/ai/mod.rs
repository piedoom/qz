mod requirement;
pub mod task;

use bevy::prelude::*;
pub use requirement::Requirement;
pub use task::TaskLabel;

/// Tracks things in a specified range
#[derive(Component, Reflect)]
pub struct InRange {
    /// Radius of the range to find objects
    pub range: f32,
    /// All allied entities in range
    pub allies: Vec<Entity>,
    /// All enemy entities in range
    pub enemies: Vec<Entity>,
}

impl InRange {
    /// Create a new empty `InRange` component from a starting range
    pub fn new(range: f32) -> Self {
        Self { range, ..default() }
    }
    /// Remove all stored `allies` and `enemies`, effectively resetting this component. This is needed whenever calculating
    /// a new frame so that old entities are not retained.
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
/// Describes a dynamic or static position in the world
#[derive(Component, Reflect)]
pub enum Waypoint {
    /// Build a waypoint set to the dynamic `translation` of the specified `Entity`
    Entity(Entity),
    /// Build a waypoint set to a static `Vec2` position
    Position(Vec2),
    /// The waypoint is not doing anything
    None,
}
