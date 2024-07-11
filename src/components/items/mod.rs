mod drop;
mod energy;
mod inventory;
mod repair;
mod weapon;

pub use {drop::*, energy::*, inventory::*, repair::*, weapon::*};

use {
    bevy::prelude::*,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Component, Reflect, Asset, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub mass: f32,
    pub size: usize,
    pub equipment: Option<EquipmentType>,
}

/// Item(s) in the world. Uses an inventory for item management
#[derive(Component, Reflect, Clone)]
pub struct Chest;

/// Tracks the chests in range for a particular entity so that the inventory of chests can become available
#[derive(Component, Reflect, Clone)]
pub struct ChestsInRange {
    pub chests: Vec<Entity>,
    pub range: f32,
}

#[derive(Debug, Reflect, Clone, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon(Weapon),
    RepairBot(RepairBot),
    Energy(Energy),
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
