use bevy::prelude::*;

use crate::prelude::*;

#[derive(Event)]
pub enum DockEvent {
    Dock { to_dock: Entity, dock: Entity },
    Undock { to_undock: Entity },
}

/// Equip and unequip items. By equipping an item, it is added as a child of the given `Entity` and
/// can then be queried normally. When unequipped, the child entity is destroyed. You may equip multiple
/// equipment items of the same type.
/// Set `manage_inventory` to false to disable shuffling inventory and equipment components, useful for initialization.
/// It is assumed that there is both an [`Inventory`] and [`Equipment`] component on the given `Entity`.
#[derive(Event)]
pub enum EquipEvent {
    Equip {
        entity: Entity,
        item: Handle<Item>,
        transfer_from_inventory: bool,
    },
    Unequip {
        entity: Entity,
        equipment: Entity,
        transfer_into_inventory: bool,
    },
}

#[derive(Event)]
pub enum InventoryEvent {
    Transfer {
        from: Entity,
        to: Entity,
        item: Handle<Item>,
        amount: usize,
    },
    TransferAll {
        from: Entity,
        to: Entity,
    },
    TossOverboard {
        entity: Entity,
        item: Handle<Item>,
        amount: usize,
    },
}

#[derive(Event)]
pub enum WorldEvent {
    SpawnCreature {
        name: String,
        slice: Slice,
        translation: Vec2,
        rotation: f32,

        alliegance: Alliegance,
        from: Option<Entity>,
    },
    SpawnBuilding {
        name: String,
        slice: Slice,
        translation: Vec2,
        rotation: f32,

        alliegance: Alliegance,
    },
    SpawnSlice(Slice),
    SpawnGate {
        from: Slice,
        to: Slice,
        translation: Vec2,
        radius: f32,
    },
}

#[derive(Event)]
pub enum StoreEvent {
    Buy {
        buyer: Entity,
        store: Entity,
        item: Handle<Item>,
        quantity: usize,
        /// Price per unit
        price: usize,
    },
    Sell {
        seller: Entity,
        store: Entity,
        item: Handle<Item>,
        quantity: usize,
        /// Price per unit
        price: usize,
    },
}
