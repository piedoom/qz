/// Money handling
mod credits;
/// Dropping items on destruction
mod drop;
/// Energy management
mod energy;
/// Equipment management
mod equipment;
/// Inventory management
mod inventory;
/// Repair bots, health, and armor
mod repair;
/// Weapons
mod weapon;

pub use {credits::*, drop::*, energy::*, equipment::*, inventory::*, repair::*, weapon::*};

use {
    bevy::prelude::*,
    serde::{Deserialize, Serialize},
};
/// A general in-game item
#[derive(Debug, Component, Clone, Reflect, Asset, Serialize, Deserialize)]
pub struct Item {
    /// The in-game name of this item
    pub name: String,
    /// The in-world mass of this item
    pub mass: f32,
    /// The space this item displaces in the [`Inventory`]
    pub size: usize,
    /// The worth of this item in `Credits`
    pub value: usize,
    /// Whether or not this item is equippable, and its associated data. If `Some(_)`, see [`EquipmentType`]
    #[serde(default)]
    pub equipment: Option<EquipmentType>,
}

/// Item(s) in the world. Uses an inventory for item management
#[derive(Component, Reflect, Clone)]
pub struct Chest;

/// Tracks the chests in range for a particular entity so that the inventory of chests can become available
#[derive(Component, Reflect, Clone)]
pub struct ChestsInRange {
    /// All chests within this range
    pub chests: Vec<Entity>,
    /// The range of this sensor
    pub range: f32,
}

impl Item {
    /// Get the item type as a string
    pub fn equipment_type_str(&self) -> &'static str {
        match &self.equipment {
            Some(eq) => match eq {
                EquipmentType::Weapon(_) => "weapon",
                EquipmentType::RepairBot(_) => "repair bot",
                EquipmentType::Generator(_) => "generator",
                EquipmentType::Battery(_) => "battery",
                EquipmentType::Armor(_) => "armor",
            },
            None => "item",
        }
    }
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
