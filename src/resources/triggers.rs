use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::prelude::*;
/// Spawn a creature in the world
#[derive(Event)]
pub struct SpawnCreature {
    /// Creature name string
    pub name: String,
    /// Spawn translation
    pub translation: Vec2,
    /// Spawn rotation in radians
    pub rotation: f32,
    /// Alliegance
    pub alliegance: Alliegance,
    /// Spawner, if applicable
    pub spawner: Option<Entity>,
}

/// Spawn a building
#[derive(Event, Serialize, Deserialize, Reflect, Clone)]
pub struct SpawnBuilding {
    /// Building name string
    pub name: String,
    /// Spawn translation
    pub translation: Vec2,
    /// Spawn rotation
    pub rotation: f32,
    /// Alliegance
    pub alliegance: Alliegance,
}

/// Generate a new chunk
#[derive(Event)]
pub struct GenerateChunks {
    /// Generate at this chunk position
    pub chunk_indicies: Vec<ChunkIndex>,
}

/// Transfer items from one inventory into another
#[derive(Event, Clone)]
pub struct InventoryTransfer {
    /// Inventory to transfer items from
    pub from: Entity,
    /// Inventory to transfer items to
    pub to: Entity,
    /// Item to transfer and its quantity, or set to transfer everything
    pub transfer: InventoryTransferSettings,
}

#[derive(Clone)]
pub enum InventoryTransferSettings {
    /// Transfer a specific item
    Item {
        /// Handle of the item to transfer
        item: Handle<Item>,
        /// Amount of this type of item to transfer
        quantity: usize,
    },
    /// Transfer all of one inventory into another
    All,
}

/// Remove items from the inventory and create a chest in the world
#[derive(Event, Clone)]
pub struct TossItemOverboard {
    /// The entity managing equipment
    pub entity: Entity,
    /// The handle of the item to throw overboard
    pub item: Handle<Item>,
    /// The amount of the selected item to throw overboard
    pub quantity: usize,
}

/**
Equip and unequip items. By equipping an item, it is added as a child of the given `Entity` and
can then be queried normally. When unequipped, the child entity is destroyed. You may equip multiple
equipment items of the same type.
*/
#[derive(Event)]
pub struct Equip {
    /// The entity to equip onto (Should have an `Equipped` component)
    pub entity: Entity,
    /// The handle of the [`Item`] to equip
    pub item: Handle<Item>,
    /// If `true`, we will transfer this item out of the inventory and error if it does not exist
    pub transfer_from_inventory: bool,
}

/// Unequip an item
#[derive(Event)]
pub struct Unequip {
    /// The child [`Equipment`] entity
    pub equipment: Entity,
    /// If `true`, we will transfer this item back into the inventory
    pub transfer_into_inventory: bool,
}
