use std::{any::TypeId, time::Duration};

use bevy::{prelude::*, utils::HashMap};
use thiserror::Error;

/// only `items` count towards the `max_size`. Equipment does not affect this.
#[derive(Default, Component)]
pub struct Inventory<'a> {
    pub max_size: usize,
    /// Starts at max_size and decrements with every item
    space: usize,
    items: HashMap<String, (&'a dyn Item, usize)>,
}

impl<'a> Inventory<'a> {
    /// Add an item to the inventory and return the new amount of that item, or an error if there is no space
    pub fn add(&mut self, item: &'a dyn Item, amount: usize) -> Result<usize, InventoryError> {
        // First, ensure we can handle the space
        let total_size = item.size() * amount;
        if self.space < total_size {
            Err(InventoryError::NoSpaceLeft {
                item_name: item.name().to_string(),
                overage: total_size - self.space,
            })
        } else {
            // We can handle the space - add items to the inventory
            let key = item.name();
            Ok(match self.items.get_mut(key) {
                Some((_, existing_amount)) => {
                    let new_amount = *existing_amount + amount;
                    *existing_amount = new_amount;
                    new_amount
                }
                None => {
                    self.items.insert(key.to_string(), (item, amount));
                    amount
                }
            })
        }
    }

    /// Remove from inventory. If no amount is specified, all items will be removed.
    pub fn remove(
        &mut self,
        item_name: &str,
        amount: Option<usize>,
    ) -> Result<(&dyn Item, usize), InventoryError> {
        match self.items.get_mut(item_name) {
            Some((item, existing_amount)) => {
                // ensure the existing amount is more than the desired amount, if specified. In unspecified, we remove everything
                if let Some(amount) = amount {
                    if amount > *existing_amount {
                        return Err(InventoryError::InsufficientItems {
                            want_to_remove: amount,
                            exists: *existing_amount,
                            item_name: item_name.to_string(),
                        });
                    }
                }

                let item = *item;

                // Get our amount or take everything
                let amount = amount.unwrap_or(*existing_amount);

                let new_amount = *existing_amount - amount;
                *existing_amount = new_amount;

                // If the existing amount is now 0, remove the item from the inventory entirely
                if new_amount == 0 {
                    self.items.remove(item_name);
                }

                Ok((item, amount))
            }
            None => {
                // No items exist in the inventory to remove
                Err(InventoryError::InsufficientItems {
                    want_to_remove: amount.unwrap_or(1),
                    exists: 0,
                    item_name: item_name.to_string(),
                })
            }
        }
    }

    pub fn count(&self, item_name: &str) -> usize {
        self.items.get(item_name).map(|x| x.1).unwrap_or_default()
    }

    /// Move items of a type to another inventory
    pub fn move_to(
        &'a mut self,
        item_name: &str,
        inventory: &mut Inventory<'a>,
        amount: Option<usize>,
    ) -> Result<(&dyn Item, usize), InventoryError> {
        let (item, amount) = self.remove(item_name, amount)?;
        inventory.add(item, amount)?;
        Ok((item, amount))
    }
}

/// Equipment needs to use the parent/child tree, so it should
/// be attached to a child entity of the main entity specifically for equipment.
/// For example, a parent `Craft` entity contains a child `Equipment` entity, and
/// that child contains multiple equipped `Items` as children of its own. This allows
/// for multiple equips of the same type to be used at once
#[derive(Default, Component)]
pub struct Equipment<'a> {
    pub inventory: Inventory<'a>,
}

impl<'a> Equipment<'a> {
    /// Equip from a general inventory into this inventory
    pub fn equip(
        &mut self,
        cmd: &mut Commands,
        equipment_entity: Entity,
        item_name: &str,
        inventory: &'a mut Inventory<'a>,
    ) -> Result<(), InventoryError> {
        // Attempt to move the inventory
        let (item, _) = inventory.move_to(item_name, &mut self.inventory, Some(1))?;
        // Add the entity to our equipment entity list
        cmd.entity(equipment_entity).with_children(|cmd| {
            item
            // cmd.spawn((item.downcast(),));
        });
        Ok(())
    }

    /// Unequip from our equipment into our general inventory
    pub fn unequip(
        &'a mut self,
        item_name: &str,
        inventory: &'a mut Inventory<'a>,
    ) -> Result<(), InventoryError> {
        // Attempt to move the inventory
        self.inventory.move_to(item_name, inventory, Some(1))?;
        Ok(())
    }
}

/// A generic item to be stored in an inventory
pub trait Item: Reflect {
    /// Item name must be unique as it is used as an ID
    fn name(&self) -> &str;
    /// Additional mass of this item
    fn mass(&self) -> f32 {
        1f32
    }
    /// Inventory slot size
    fn size(&self) -> usize {
        1
    }
    /// Category of this item
    fn category(&self) -> ItemCategory;
    fn type_id(&self) -> TypeId;
}

pub trait Weapon: Item {}

#[derive(Reflect)]
pub struct ProjectileWeapon {
    pub name: String,
    /// Duration between new projectile shots
    pub recoil: Duration,
    /// Projectile to clone
    pub projectile: Projectile,
}

#[derive(Reflect)]
pub struct Projectile {
    pub speed: f32,
    pub damage: usize,
}

impl Item for ProjectileWeapon {
    fn name(&self) -> &str {
        &self.name
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Weapon
    }
    fn type_id(&self) -> TypeId {
        TypeId::of::<ProjectileWeapon>()
    }
}

impl Weapon for ProjectileWeapon {}

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
}
