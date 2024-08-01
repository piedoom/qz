use bevy::{ecs::query::QueryEntityError, prelude::*};
use thiserror::Error;

use crate::prelude::*;

/// A catch-all error type for game event errors,
/// like inventory space requirement failures. This is also a bevy event type,
/// so we can read errors like any other event
#[derive(Event, Error, Debug)]
pub enum GameError {
    #[error(transparent)]
    InventoryError(#[from] InventoryError),

    #[error(transparent)]
    EquipmentError(#[from] EquipmentError),

    #[error(transparent)]
    WorldEventError(#[from] WorldEventError),

    #[error(transparent)]
    StoreError(#[from] StoreError),
}

#[derive(Error, Debug)]
pub enum EnergyError {
    #[error("requested `{requested}` energy when only `{actual}` is available")]
    InsufficientCharge { requested: f32, actual: f32 },
}

#[derive(Error, Debug)]
pub enum WorldEventError {
    #[error("could not find asset with key {0}")]
    AssetNotFound(String),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum InventoryError {
    #[error("adding item(s) to the inventory would exceed the maximum space by `{overage}`")]
    NoSpaceLeft { overage: usize },
    #[error("attempted to remove `{want_to_remove}` when `{exists}` exists")]
    InsufficientItems {
        want_to_remove: usize,
        exists: usize,
    },
    #[error("attempted to equip unequippable item `{item_name}`")]
    Unequippable { item_name: String },
    #[error("missing either an `Inventory` or `Equipment` component on the provided entity")]
    Unqueriable,
    #[error("could not find requested item with handle")]
    ItemNotFound,
    #[error(transparent)]
    QueryEntityError(#[from] QueryEntityError),
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(transparent)]
    QueryEntityError(#[from] QueryEntityError),
    #[error(transparent)]
    CreditsError(#[from] CreditsError),
    #[error(transparent)]
    InventoryError(#[from] InventoryError),
    #[error("not enough items")]
    NotEnoughItems,
    #[error("not enough credits")]
    NotEnoughCredits,
}

#[derive(Debug, Error)]
pub enum EquipmentError {
    #[error(transparent)]
    QueryEntityError(#[from] QueryEntityError),
    #[error(transparent)]
    InventoryError(#[from] InventoryError),
    #[error("slot not available")]
    SlotNotAvailable,
}
