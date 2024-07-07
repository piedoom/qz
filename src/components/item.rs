use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use thiserror::Error;

use crate::resources::events::EquipEvent;

/// only `items` count towards the `max_size`. Equipment does not affect this.
#[derive(Component)]
pub struct Inventory {
    pub max_size: usize,
    /// Starts at max_size and decrements with every item
    space: usize,
    pub items: HashMap<Item, usize>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            max_size: 64,
            space: 64,
            items: HashMap::default(),
        }
    }
}

impl Inventory {
    pub fn with(mut self, item: Item, amount: usize) -> Result<Self, InventoryError> {
        self.add(item, amount)?;
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

    pub fn count(&self, item: &Item) -> usize {
        self.items.get(item).cloned().unwrap_or_default()
    }

    /// Move items of a type to another inventory
    pub fn move_to(
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

/// Equipment needs to use the parent/child tree, so it should
/// be attached to a child entity of the main entity specifically for equipment.
/// For example, a parent `Craft` entity contains a child `Equipment` entity, and
/// that child contains multiple equipped `Items` as children of its own. This allows
/// for multiple equips of the same type to be used at once
#[derive(Default, Component)]
pub struct Equipment {
    pub inventory: Inventory,
}

#[derive(Clone, Component)]
pub struct Item {
    pub name: &'static str,
    pub mass: f32,
    pub size: usize,
    pub equipment: Option<EquipmentType>,
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Item {}

impl std::hash::Hash for Item {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

#[derive(Clone)]
pub enum EquipmentType {
    Weapon(Weapon),
}

#[derive(Clone, Component)]
pub struct Weapon {
    pub wants_to_fire: bool,
    pub last_fired: Duration,
    pub weapon_type: WeaponType,
}

#[derive(Clone)]
pub enum WeaponType {
    Projectile {
        /// Speed of projectile
        speed: f32,
        /// Duration between new projectile shots
        recoil: Duration,
        // Cone in radians of potential spread
        spread: f32,
        // Shots to fire at once
        shots: usize,
        damage: usize,
        radius: f32,
        lifetime: Duration,
    },
}

#[derive(Clone, Component)]
pub struct Projectile {
    pub damage: usize,
}

/// Marker for the item type used for determining equippable status as well as for categorization
/// This also allows us to implement multiple categories of items if we'd like but still have a single item category
pub enum ItemCategory {
    Weapon,
    None,
}

impl ItemCategory {
    pub fn equippable(&self) -> bool {
        match self {
            ItemCategory::Weapon => true,
            ItemCategory::None => false,
        }
    }
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
}
