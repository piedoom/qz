mod credits;
mod drop;
mod energy;
mod inventory;
mod repair;
mod weapon;

pub use {credits::*, drop::*, energy::*, inventory::*, repair::*, weapon::*};

use {
    bevy::prelude::*,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Component, Reflect, Asset, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub mass: f32,
    pub size: usize,
    pub value: usize,
    #[serde(default)]
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
    Generator(Generator),
    Battery(Battery),
    Armor(Armor),
}

impl Item {
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
