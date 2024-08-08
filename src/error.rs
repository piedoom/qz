use bevy::{ecs::query::QueryEntityError, prelude::*};
use thiserror::Error;

use crate::prelude::*;

/// A catch-all error type for game event errors,
/// like inventory space requirement failures. This is also a bevy event type,
/// so we can read errors like any other event
#[derive(Event, Error, Debug)]
pub enum GameError {
    /// Inventory error
    #[error(transparent)]
    InventoryError(#[from] InventoryError),
    /// Equipment error
    #[error(transparent)]
    EquipmentError(#[from] EquipmentError),
    /// World event error
    #[error(transparent)]
    WorldEventError(#[from] WorldEventError),
    /// Store error
    #[error(transparent)]
    StoreError(#[from] StoreError),
}
/// Energy error
#[derive(Error, Debug)]
pub enum EnergyError {
    /// Cannot satisfy energy request
    #[error("requested `{requested}` energy when only `{actual}` is available")]
    InsufficientCharge {
        /// Amount of energy requested
        requested: f32,
        /// Amount of energy available
        actual: f32,
    },
}

/// World event error
#[derive(Error, Debug)]
pub enum WorldEventError {
    /// Asset not found with a string key
    #[error("could not find asset with key {0}")]
    AssetNotFound(String),
}

/// Inventory error
#[derive(Error, Debug, PartialEq, Eq)]
pub enum InventoryError {
    /// No space left in the inventory
    #[error("adding item(s) to the inventory would exceed the maximum space by `{overage}`")]
    NoSpaceLeft {
        /// Space over the maximum
        overage: usize,
    },
    /// More items requested than this inventory contains
    #[error("attempted to remove `{want_to_remove}` when `{exists}` exists")]
    InsufficientItems {
        /// Wanted to remove
        want_to_remove: usize,
        /// Number that exists in inventory
        exists: usize,
    },
    /// Attempted to equip an unequippable item
    #[error("attempted to equip unequippable item `{item_name}`")]
    Unequippable {
        /// Item name string
        item_name: String,
    },
    /// Unable to get the necessary [`Inventory`] or [`Equipment`] components of the provided entity
    #[error("missing either an `Inventory` or `Equipment` component on the provided entity")]
    Unqueriable,
    /// No item found with given handle
    #[error("could not find requested item with handle")]
    ItemNotFound,
    /// Query entity error
    #[error(transparent)]
    QueryEntityError(#[from] QueryEntityError),
}

/// Store error
#[derive(Debug, Error)]
pub enum StoreError {
    /// Query entity error
    #[error(transparent)]
    QueryEntityError(#[from] QueryEntityError),
    /// Credits error
    #[error(transparent)]
    CreditsError(#[from] CreditsError),
    /// Inventory error
    #[error(transparent)]
    InventoryError(#[from] InventoryError),
    /// Not enough items in inventory
    #[error("not enough items")]
    NotEnoughItems,
    /// Not enough credits
    #[error("not enough credits")]
    NotEnoughCredits,
}

/// Equipment error
#[derive(Debug, Error)]
pub enum EquipmentError {
    /// Query entity error
    #[error(transparent)]
    QueryEntityError(#[from] QueryEntityError),
    /// Inventory error
    #[error(transparent)]
    InventoryError(#[from] InventoryError),
    /// Slot not available (full or does not exist)
    #[error("slot not available")]
    SlotNotAvailable,
}
