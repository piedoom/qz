use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prelude::*;

/// only `items` count towards the `max_size`. Equipment does not affect this.
#[derive(Debug, Clone, Component, Reflect, Serialize, Deserialize)]
pub struct Inventory {
    /// Maximum size of the inventory, determined by the craft (so it is not serialized)
    #[serde(default)]
    pub capacity: usize,
    /// Starts at max_size and decrements with every item
    #[serde(default)]
    space: usize,
    pub items: HashMap<Item, usize>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            capacity: 64,
            // TODO: Make this work
            space: 64,
            items: HashMap::default(),
        }
    }
}

impl Inventory {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            ..default()
        }
    }

    pub fn with(mut self, item: Item, amount: usize) -> Result<Self, InventoryError> {
        self.add(item, amount)?;
        Ok(self)
    }

    pub fn with_many(
        mut self,
        items_amounts: HashMap<String, usize>,
        items: &Assets<Item>,
        library: &Library,
    ) -> Result<Self, InventoryError> {
        for (item_name, amount) in items_amounts.iter() {
            item(item_name, items, library)
                .map(|x| self.add(x.clone(), *amount))
                .ok_or(InventoryError::ItemNotFound {
                    name: item_name.to_string(),
                })
                .flatten()?;
        }
        Ok(self)
    }

    pub fn with_many_single(
        mut self,
        item_names: &[&'static str],
        items: &Assets<Item>,
        library: &Library,
    ) -> Result<Self, InventoryError> {
        for item_name in item_names.iter() {
            item(item_name, items, library)
                .map(|x| self.add(x.clone(), 1))
                .ok_or(InventoryError::ItemNotFound {
                    name: item_name.to_string(),
                })
                .flatten()?;
        }
        Ok(self)
    }

    /// Add an item to the inventory and return the new amount of that item, or an error if there is no space
    pub fn add(&mut self, item: Item, amount: usize) -> Result<usize, InventoryError> {
        // First, ensure we can handle the space
        let total_size = item.size * amount;
        if self.space < total_size {
            Err(InventoryError::NoSpaceLeft {
                item_name: item.name.to_string(),
                overage: total_size - self.space,
            })
        } else {
            // We can handle the space - add items to the inventory
            Ok(if let Some(existing_amount) = self.items.get_mut(&item) {
                let new_amount = *existing_amount + amount;
                *existing_amount = new_amount;
                new_amount
            } else {
                self.items.insert(item, amount);
                amount
            })
        }
    }

    /// Remove from inventory. If no amount is specified, all items will be removed.
    pub fn remove(&mut self, item: &Item, amount: Option<usize>) -> Result<usize, InventoryError> {
        match self.items.get_mut(item) {
            Some(existing_amount) => {
                // ensure the existing amount is more than the desired amount, if specified. In unspecified, we remove everything
                if let Some(amount) = amount {
                    if amount > *existing_amount {
                        return Err(InventoryError::InsufficientItems {
                            want_to_remove: amount,
                            exists: *existing_amount,
                            item_name: item.name.to_string(),
                        });
                    }
                }

                // let item = item.clone();

                // Get our amount or take everything
                let amount = amount.unwrap_or(*existing_amount);

                let new_amount = *existing_amount - amount;
                *existing_amount = new_amount;

                // If the existing amount is now 0, remove the item from the inventory entirely
                if new_amount == 0 {
                    self.items.remove(item);
                }

                Ok(amount)
            }
            None => {
                // No items exist in the inventory to remove
                Err(InventoryError::InsufficientItems {
                    want_to_remove: amount.unwrap_or(1),
                    exists: 0,
                    item_name: item.name.to_string(),
                })
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn count(&self, item: &Item) -> usize {
        self.items.get(item).cloned().unwrap_or_default()
    }

    /// Move items of a type to another inventory
    pub fn transfer(
        &mut self,
        item: &Item,
        inventory: &mut Inventory,
        amount: Option<usize>,
    ) -> Result<usize, InventoryError> {
        let amount = self.remove(item, amount)?;
        inventory.add(item.clone(), amount)?;
        Ok(amount)
    }
}

/// Equipment needs to use the parent/child tree. This allows
/// for multiple equips of the same type to be used at once
#[derive(Default, Component, Reflect)]
pub struct Equipment {
    pub inventory: Inventory,
}

#[derive(Error, Debug)]
pub enum InventoryError {
    #[error(
        "adding `{item_name}`  to the inventory would exceed the maximum space by `{overage}`"
    )]
    NoSpaceLeft { item_name: String, overage: usize },
    #[error("attempted to remove `{want_to_remove}` of `{item_name}` when only {exists} exists")]
    InsufficientItems {
        want_to_remove: usize,
        exists: usize,
        item_name: String,
    },
    #[error("attempted to equip unequippable item `{item_name}`")]
    Unequippable { item_name: String },
    #[error("missing either an `Inventory` or `Equipment` component on the provided entity")]
    Unqueriable,
    #[error("could not find requested item {name}")]
    ItemNotFound { name: String },
}
