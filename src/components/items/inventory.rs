use bevy::{
    prelude::*,
    utils::{hashbrown, HashMap},
};
use serde::{Deserialize, Serialize};

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

    // TODO: make this work with serde
    #[serde(skip)]
    items: HashMap<Handle<Item>, usize>,
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
    pub fn quantity(&self, item: &Handle<Item>) -> usize {
        self.items.get(item).cloned().unwrap_or_default()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn space_occupied(&self) -> usize {
        self.space_occupied
    }

    pub fn space_remaining(&self) -> usize {
        self.capacity - self.space_occupied
    }

    pub fn iter(&self) -> hashbrown::hash_map::Iter<Handle<Item>, usize> {
        self.items.iter()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            ..default()
        }
    }

    pub fn with(
        mut self,
        item: Handle<Item>,
        amount: usize,
        items: &Assets<Item>,
    ) -> Result<Self, InventoryError> {
        self.add(item, amount, items)?;
        Ok(self)
    }

    /// Add an item to the inventory
    pub fn add(
        &mut self,
        item: Handle<Item>,
        amount: usize,
        items: &Assets<Item>,
    ) -> Result<(), InventoryError> {
        // Retrieve the item from storage
        let retrieved_item = items.get(&item).ok_or(InventoryError::ItemNotFound)?;
        // Ensure we can handle the space
        let total_size = retrieved_item.size * amount;
        if self.space_occupied + total_size > self.capacity() {
            Err(InventoryError::NoSpaceLeft {
                overage: self.space_occupied + total_size - self.capacity,
            })
        } else {
            // We can handle the space
            // Add the size of the added items to the inventory's tracker so we don't need to calculate
            // this every frame
            self.space_occupied += total_size;
            // Add items to the inventory. If we already have an item of the type,
            // get it from our hashmap and add. Otherwise, insert.
            if let Some(existing_amount) = self.items.get_mut(&item) {
                let new_amount = *existing_amount + amount;
                *existing_amount = new_amount;
            } else {
                self.items.insert(item, amount);
            };
            Ok(())
        }
    }

    pub fn remove(
        &mut self,
        item: Handle<Item>,
        amount: usize,
        items: &Assets<Item>,
    ) -> Result<(), InventoryError> {
        // Retrieve the item from storage
        let retrieved_item = items.get(&item).ok_or(InventoryError::ItemNotFound)?;
        match self.items.get_mut(&item) {
            Some(existing_amount) => {
                // ensure the existing amount is more than the desired amount, if specified. In unspecified, we remove everything
                if amount > *existing_amount {
                    return Err(InventoryError::InsufficientItems {
                        want_to_remove: amount,
                        exists: *existing_amount,
                        item_name: retrieved_item.name.to_string(),
                    });
                }

                // Subtrack from our inventory space
                self.space_occupied -= amount * retrieved_item.size;

                let new_amount = *existing_amount - amount;
                *existing_amount = new_amount;

                // If the existing amount is now 0, remove the item from the inventory entirely
                if new_amount == 0 {
                    self.items.remove(&item);
                }

                Ok(())
            }
            None => {
                // No items exist in the inventory to remove
                Err(InventoryError::InsufficientItems {
                    want_to_remove: amount,
                    exists: 0,
                    item_name: retrieved_item.name.to_string(),
                })
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn count(&self, item: Handle<Item>) -> usize {
        self.items.get(&item).cloned().unwrap_or_default()
    }

    /// Move items of a type to another inventory
    pub fn transfer(
        &mut self,
        item: Handle<Item>,
        inventory: &mut Inventory,
        amount: usize,
        items: &Assets<Item>,
    ) -> Result<usize, InventoryError> {
        inventory.add(item.clone(), amount, items)?;
        self.remove(item.clone(), amount, items)?;
        Ok(amount)
    }

    /// Move all items into another inventory
    pub fn transfer_all(&mut self, inventory: &mut Inventory) -> Result<(), InventoryError> {
        // Short circuit if not enough space in new inventory
        if self.space_occupied() > inventory.space_remaining() {
            return Err(InventoryError::NoSpaceLeft {
                overage: self.space_occupied() - inventory.space_remaining(),
            });
        }

        inventory.space_occupied += self.space_occupied;
        self.space_occupied = 0;

        // Drain into provided inventory
        inventory.items.extend(self.items.drain());

        Ok(())
    }

    pub fn with_many_from_str(
        mut self,
        hash_map: HashMap<String, usize>,
        items: &Assets<Item>,
        library: &Library,
    ) -> Result<Self, InventoryError> {
        for (k, v) in hash_map.iter() {
            let item = library
                .items
                .get(&format!("items/{}.ron", k))
                .ok_or(InventoryError::ItemNotFound)?;
            self.add(item.clone(), *v, items)?;
        }
        Ok(self)
    }
}

/// Equipment needs to use the parent/child tree. This allows
/// for multiple equips of the same type to be used at once
#[derive(Default, Component, Reflect)]
pub struct Equipment {
    pub inventory: Inventory,
}

// #[cfg(test)]
// mod tests {
//     use std::assert_matches::assert_matches;

//     use super::*;

//     fn item(size: usize) -> Item {
//         Item {
//             name: "item name".to_string(),
//             mass: 10.0,
//             size,
//             equipment: None,
//         }
//     }

//     /// Successfully adds an item to the inventory
//     #[test]
//     fn add() {
//         let mut inv = Inventory::with_capacity(100);
//         assert_matches!(inv.add(item(1), 1), Ok(()));
//     }

//     /// Successfully removes an item from the inventory
//     #[test]
//     fn remove() {
//         let mut inv = Inventory::with_capacity(100);
//         inv.add(item(1), 1).unwrap();
//         assert_matches!(inv.remove(&item(1), Some(1)), Ok(1));
//     }

//     /// Successfully adds multiple items to the inventory
//     #[test]
//     fn advanced_add() {
//         let mut inv = Inventory::with_capacity(10);
//         assert_matches!(inv.add(item(2), 4), Ok(()));
//     }

//     // /// Successfully removes multiple items from the inventory
//     // #[test]
//     // fn advanced_remove() {
//     //     let mut inv = Inventory::with_capacity(100);
//     //     inv.add(item(1), 1).unwrap();
//     //     assert_matches!(inv.remove(&item(1), Some(1)), Ok(1));
//     // }

//     /// Unsuccessfully attempts to add an item to a full inventory
//     #[test]
//     fn unsuccessful_add() {
//         let mut inv = Inventory::with_capacity(1);
//         inv.add(item(1), 1).unwrap();
//         assert_matches!(inv.add(item(1), 1), Err(_));
//     }

//     /// Unsuccessfully attempts to remove more items than exists in the inventory
//     #[test]
//     fn unsuccessful_remove() {
//         let mut inv = Inventory::with_capacity(100);
//         inv.add(item(1), 2).unwrap();
//         inv.remove(&item(1), Some(1)).unwrap();
//         assert_matches!(inv.remove(&item(1), Some(2)), Err(_));
//     }
// }
