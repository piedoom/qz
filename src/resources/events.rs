use bevy::prelude::*;

use crate::prelude::*;

/// Equip and unequip items. By equipping an item, it is added as a child of the given `Entity` and
/// can then be queried normally. When unequipped, the child entity is destroyed. You may equip multiple
/// equipment items of the same type.
/// Set `manage_inventory` to false to disable shuffling inventory and equipment components, useful for initialization.
/// It is assumed that there is both an [`Inventory`] and [`Equipment`] component on the given `Entity`.
#[derive(Event)]
pub enum EquipEvent {
    Equip {
        entity: Entity,
        item: Item,
        manage_inventory: bool,
    },
    Unequip {
        entity: Entity,
        item: Item,
        manage_inventory: bool,
    },
}
