use std::usize;

use avian3d::prelude::*;
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
    pub fn max_capacity() -> Self {
        Self {
            capacity: usize::MAX,
            space_occupied: 0,
            items: HashMap::new(),
        }
    }

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
        let retrieved_item = items.get(&item).ok_or(InventoryError::ItemNotFound)?;
        self.add(item, retrieved_item.size, amount)?;
        Ok(self)
    }

    /// Add an item to the inventory
    pub fn add(
        &mut self,
        item: Handle<Item>,
        size: usize,
        amount: usize,
    ) -> Result<(), InventoryError> {
        // Ensure we can handle the space
        let total_size = size * amount;
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
        size: usize,
        amount: usize,
    ) -> Result<(), InventoryError> {
        // Retrieve the item from storage
        match self.items.get_mut(&item) {
            Some(existing_amount) => {
                // ensure the existing amount is more than the desired amount, if specified. In unspecified, we remove everything
                if amount > *existing_amount {
                    return Err(InventoryError::InsufficientItems {
                        want_to_remove: amount,
                        exists: *existing_amount,
                    });
                }

                // Subtrack from our inventory space
                self.space_occupied -= amount * size;

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
                })
            }
        }
    }

    pub fn mass(&self, items: &Assets<Item>) -> f32 {
        self.items.iter().fold(0f32, |a, (handle, amt)| {
            a + (items.get(handle).unwrap().mass * *amt as f32)
        })
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
        let retrieved_item = items.get(&item).ok_or(InventoryError::ItemNotFound)?;
        inventory.add(item.clone(), retrieved_item.size, amount)?;
        self.remove(item.clone(), retrieved_item.size, amount)?;
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
        for (item, amount) in self.items.drain() {
            if let Some(existing_amount) = inventory.items.get_mut(&item) {
                *existing_amount += amount
            } else {
                inventory.items.insert(item, amount);
            }
        }

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
            let retrieved_item = items.get(item).ok_or(InventoryError::ItemNotFound)?;
            self.add(item.clone(), retrieved_item.size, *v)?;
        }
        Ok(self)
    }

    pub fn drain(&mut self) -> hashbrown::hash_map::Drain<Handle<Item>, usize> {
        self.space_occupied = 0;
        self.items.drain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_add_over() {
        let mut inv = Inventory::with_capacity(10);
        assert!(inv.add(Handle::default(), 3, 1).is_ok());
        assert!(inv.add(Handle::default(), 6, 1).is_ok());
        assert!(inv
            .add(Handle::default(), 3, 1)
            .is_err_and(|x| x == InventoryError::NoSpaceLeft { overage: 2 }))
    }
}
