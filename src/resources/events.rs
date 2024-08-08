use bevy::prelude::*;
use petgraph::graph::NodeIndex;

use crate::prelude::*;

/// Dock an entity to a dock
#[derive(Event)]
pub enum DockEvent {
    /// DOck to an entity
    Dock {
        /// The entity to dock to
        to_dock: Entity,
        /// The dock entity
        dock: Entity,
    },
    /// Undock from an entity
    Undock {
        /// The entity to undock from
        to_undock: Entity,
    },
}

/// Equip and unequip items. By equipping an item, it is added as a child of the given `Entity` and
/// can then be queried normally. When unequipped, the child entity is destroyed. You may equip multiple
/// equipment items of the same type.
/// Set `manage_inventory` to false to disable shuffling inventory and equipment components, useful for initialization.
/// It is assumed that there is both an [`Inventory`] and [`Equipment`] component on the given `Entity`.
#[derive(Event)]
pub enum EquipEvent {
    /// Equip an item
    Equip {
        /// The entity to equip onto (Should have an `Equipped` component)
        entity: Entity,
        /// The handle of the [`Item`] to equip
        item: Handle<Item>,
        /// If `true`, we will transfer this item out of the inventory and error if it does not exist
        transfer_from_inventory: bool,
    },
    /// Unequip an item
    Unequip {
        /// The entity managing equipment
        entity: Entity,
        /// The child [`Equipment`] entity
        equipment: Entity,
        /// If `true`, we will transfer this item back into the inventory
        transfer_into_inventory: bool,
    },
}

/// Inventory events
#[derive(Event)]
pub enum InventoryEvent {
    /// Transfer items from one inventory into another
    Transfer {
        /// Inventory to transfer items from
        from: Entity,
        /// Inventory to transfer items to
        to: Entity,
        /// Handle of the item to transfer
        item: Handle<Item>,
        /// Amount of this type of item to transfer
        amount: usize,
    },
    /// Transfer all of one inventory into another
    TransferAll {
        /// Inventory to transfer all items from
        from: Entity,
        /// Inventory to transfer all items to
        to: Entity,
    },
    /// Remove items from the inventory and create a chest in the world
    TossOverboard {
        /// The entity managing equipment
        entity: Entity,
        /// The handle of the item to throw overboard
        item: Handle<Item>,
        /// The amount of the selected item to throw overboard
        amount: usize,
    },
}

/// Store events for buying and selling
#[derive(Event)]
pub enum StoreEvent {
    /// Buy an item
    Buy {
        /// Buyer entity
        buyer: Entity,
        /// Store entity
        store: Entity,
        /// Item to be bought by the buyer entity
        item: Handle<Item>,
        /// Quantity to be bought by the buyer entity
        quantity: usize,
        /// Price per unit
        price: usize,
    },
    /// Sell an item
    Sell {
        /// Seller entity
        seller: Entity,
        /// Store entity
        store: Entity,
        /// Item to be sold by the seller entity
        item: Handle<Item>,
        /// Quantity to be sold by the seller entity
        quantity: usize,
        /// Price per unit
        price: usize,
    },
}

/// Module for game level triggers to setup and spawn the world
pub mod triggers {
    use super::*;

    /// Spawn a creature
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
        /// Optional spawner this creature was spawned from
        pub spawner: Option<Entity>,
    }
}

/// Save the universe
#[derive(Event)]
pub struct Save;

/// Load a [`Universe`] node, or generate a new one and load it
#[derive(Event)]
pub struct Load {
    /// If None is specified, the load will attempt from the universe position
    pub node: Option<NodeIndex>,
    /// If specified, the player will be moved to the gate matching this node on entry to the zone
    pub from_node: Option<NodeIndex>,
}
