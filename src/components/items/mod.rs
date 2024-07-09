mod drop;
mod inventory;
mod repair;
mod weapon;

pub use {drop::*, inventory::*, repair::*, weapon::*};

use bevy::prelude::*;

#[derive(Clone, Component)]
pub struct Item {
    pub name: &'static str,
    pub mass: f32,
    pub size: usize,
    pub equipment: Option<EquipmentType>,
}

/// Item(s) in the world. Uses an inventory for item management
#[derive(Component, Clone)]
pub struct Chest;

/// Tracks the chests in range for a particular entity so that the inventory of chests can become available
#[derive(Component, Clone)]
pub struct ChestsInRange {
    pub chests: Vec<Entity>,
    pub range: f32,
}

#[derive(Clone)]
pub enum EquipmentType {
    Weapon(Weapon),
    RepairBot(RepairBot),
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Item {}

impl std::hash::Hash for Item {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}
