use bevy::{
    prelude::*,
    utils::{hashbrown, HashMap},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prelude::*;

/// only `items` count towards the `max_size`. Equipment does not affect this.
#[derive(Debug, Clone, Component, Reflect, Serialize, Deserialize)]
pub struct Inventory {
    /// Maximum size of the inventory, determined by the craft (so it is not serialized)
    #[serde(default)]
    capacity: usize,
    /// Spaces out of the capacity that are occupied by items
    #[serde(default)]
    space_occupied: usize,
    items: HashMap<Item, usize>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            capacity: 64,
            items: HashMap::default(),
            space_occupied: default(),
        }
    }
}

impl Inventory {
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn space_occupied(&self) -> usize {
        self.space_occupied
    }

    pub fn iter(&self) -> hashbrown::hash_map::Iter<Item, usize> {
        self.items.iter()
    }

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

    /// Add an item to the inventory
    pub fn add(&mut self, item: Item, amount: usize) -> Result<(), InventoryError> {
        // First, ensure we can handle the space
        let total_size = item.size * amount;
        if self.space_occupied >= total_size {
            Err(InventoryError::NoSpaceLeft {
                item_name: item.name.to_string(),
                overage: self.space_occupied - total_size,
            })
        } else {
            // We can handle the space
            // Add the size of the added items to the inventory's tracker so we don't need to calculate
            // this every frame
            self.space_occupied += total_size;
            // Add items to the inventory. If we already have an item of the type,
            // get it from our hashmap and add. Otherwise, insert.
            Ok(if let Some(existing_amount) = self.items.get_mut(&item) {
                let new_amount = *existing_amount + amount;
                *existing_amount = new_amount;
            } else {
                self.items.insert(item, amount);
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

                // Get our amount or take everything
                let amount_to_remove = amount.unwrap_or(*existing_amount);

                // Subtrack from our inventory space
                self.space_occupied -= amount_to_remove;

                let new_amount = *existing_amount - amount_to_remove;
                *existing_amount = new_amount;

                // If the existing amount is now 0, remove the item from the inventory entirely
                if new_amount == 0 {
                    self.items.remove(item);
                }

                Ok(amount_to_remove)
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

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    fn item(size: usize) -> Item {
        Item {
            name: "item name".to_string(),
            mass: 10.0,
            size,
            equipment: None,
        }
    }

    /// Successfully adds an item to the inventory
    #[test]
    fn add() {
        let mut inv = Inventory::with_capacity(100);
        assert_matches!(inv.add(item(1), 1), Ok(()));
    }

    /// Successfully removes an item from the inventory
    #[test]
    fn remove() {
        let mut inv = Inventory::with_capacity(100);
        inv.add(item(1), 1).unwrap();
        assert_matches!(inv.remove(&item(1), Some(1)), Ok(1));
    }

    /// Successfully adds multiple items to the inventory
    #[test]
    fn advanced_add() {
        let mut inv = Inventory::with_capacity(10);
        assert_matches!(inv.add(item(2), 4), Ok(()));
    }

    // /// Successfully removes multiple items from the inventory
    // #[test]
    // fn advanced_remove() {
    //     let mut inv = Inventory::with_capacity(100);
    //     inv.add(item(1), 1).unwrap();
    //     assert_matches!(inv.remove(&item(1), Some(1)), Ok(1));
    // }

    /// Unsuccessfully attempts to add an item to a full inventory
    #[test]
    fn unsuccessful_add() {
        let mut inv = Inventory::with_capacity(1);
        inv.add(item(1), 1).unwrap();
        assert_matches!(inv.add(item(1), 1), Err(_));
    }

    /// Unsuccessfully attempts to remove more items than exists in the inventory
    #[test]
    fn unsuccessful_remove() {
        let mut inv = Inventory::with_capacity(100);
        inv.add(item(1), 2).unwrap();
        inv.remove(&item(1), Some(1)).unwrap();
        assert_matches!(inv.remove(&item(1), Some(2)), Err(_));
    }
}
