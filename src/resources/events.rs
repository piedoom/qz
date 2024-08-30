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
    },
}

/// Save the universe
#[derive(Event, Clone, Copy)]
pub struct Save {
    pub node: NodeIndex,
}
