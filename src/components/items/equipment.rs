use std::ops::{Add, AddAssign};

use bevy::{
    ecs::{
        component::{ComponentHooks, StorageType},
        world::DeferredWorld,
    },
    prelude::*,
    utils::{
        hashbrown::{hash_map::Iter, HashMap},
        HashSet,
    },
};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Equipment needs to use the parent/child tree. This allows
/// for multiple equips of the same type to be used at once
#[derive(Debug, Component, Default, Clone, Reflect, Serialize, Deserialize)]
pub struct Equipped {
    /// A map of [`EquipmentTypeId`]s to equipped entities. This is updated via hooks and is hopefully never invalid.
    /// This value is essentially a cache.
    pub equipped: HashMap<EquipmentTypeId, HashSet<Entity>>,
    /// Defines the shape of what can be `equipped`, where `usize` is the total maximum equippable of that type.
    pub slots: HashMap<EquipmentTypeId, usize>,
    // pub inventory: Inventory,
}

impl Equipped {
    /// Iterate through all equipment
    pub fn iter(&self) -> Iter<EquipmentTypeId, HashSet<Entity>> {
        self.equipped.iter()
    }

    /// Total space
    pub fn capacity(&self, equipment_type: &EquipmentTypeId) -> usize {
        self.slots.get(equipment_type).cloned().unwrap_or_default()
    }

    /// Available space left
    pub fn slots_remaining(&self, equipment_type: &EquipmentTypeId) -> usize {
        let max = self.slots.get(equipment_type).cloned().unwrap_or_default();
        let current = self
            .equipped
            .get(equipment_type)
            .map(|x| x.len())
            .unwrap_or_default();
        max - current
    }

    /// Total mass of equipped items
    pub fn mass(&self, item_assets: &Assets<Item>, items: &Query<&Equipment>) -> f32 {
        self.equipped
            .iter()
            .flat_map(|(_, e)| e.iter())
            .fold(0f32, |acc, x| {
                acc + item_assets
                    .get(&items.get(*x).unwrap().handle())
                    .unwrap()
                    .mass
            })
    }
}

/// Build an `Equipped` with starting items
#[derive(Debug, Component, Default, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EquippedBuilder {
    /// Names of items to equip
    pub equipped: Vec<String>,
    /// Slot shape definitions
    pub slots: Vec<(EquipmentTypeId, usize)>,
}

/// Allows us to specify specific equipment categories
#[derive(Debug, Reflect, Clone, Serialize, Deserialize)]
pub enum EquipmentType {
    /// A weapon that can fire
    Weapon(Weapon),
    /// Repairs damage over time
    RepairBot(RepairBot),
    /// Generates [`Energy`]
    Generator(Generator),
    /// Stores [`Energy`]
    Battery(Battery),
    /// Increases maximum [`Health`]
    Armor(Armor),
}

/// Defines an `EquipmentType` without associated information. This should be kept in sync with `EquipmentType`.
#[derive(
    Copy, Clone, Debug, Reflect, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub enum EquipmentTypeId {
    /// A weapon that can fire
    Weapon,
    /// Repairs damage over time
    RepairBot,
    /// Generates [`Energy`]
    Generator,
    /// Stores [`Energy`]
    Battery,
    /// Increases maximum [`Health`]
    Armor,
}

impl std::fmt::Display for EquipmentTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&EquipmentType> for EquipmentTypeId {
    fn from(value: &EquipmentType) -> Self {
        match value {
            EquipmentType::Weapon(_) => Self::Weapon,
            EquipmentType::RepairBot(_) => Self::RepairBot,
            EquipmentType::Generator(_) => Self::Generator,
            EquipmentType::Battery(_) => Self::Battery,
            EquipmentType::Armor(_) => Self::Armor,
        }
    }
}

impl EquipmentType {
    /// Get the type of this equipment without any associated data
    pub fn id(&self) -> EquipmentTypeId {
        EquipmentTypeId::from(self)
    }
}

/// Wrapper component for a handle that is guarenteed to be an equipment item.
/// Inserting these as a child with [`Equipped`] will trigger equipment management.
#[derive(Debug, Reflect, Clone)]
pub struct Equipment(Handle<Item>);

impl Equipment {
    /// Obtain the handle of this [`Item`]
    pub fn handle(&self) -> Handle<Item> {
        self.0.clone()
    }

    /// Crate a new [`Equipment`] from an existing [`Item`] handle
    pub fn new(item_handle: Handle<Item>) -> Self {
        Self(item_handle)
    }
}

impl Component for Equipment {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    // When adding an item as a component...
    fn register_component_hooks(hooks: &mut ComponentHooks) {
        /// Helper function since we do basically the same thing on add and remove except for a minus sign
        fn modify(world: &mut DeferredWorld, entity: Entity, add: bool) {
            if let (Some(parent), Some(equipment)) = (
                world.get::<Parent>(entity).map(|p| p.get()),
                world.get::<Equipment>(entity).cloned(),
            ) {
                let items = world.get_resource::<Assets<Item>>().unwrap();
                let retrieved_item = items.get(&equipment.0).unwrap().clone();

                if let Some(mut equipped) = world.get_mut::<Equipped>(parent) {
                    // We have all necessary variables!
                    // NOTE: We assume that there is a slot available in equipment at this point - it should
                    // be checked before adding.
                    if add {
                        let equipment_type = retrieved_item.equipment.clone().unwrap();

                        // Register or add the entity in the equipped
                        let id = equipment_type.id();
                        if equipped.equipped.contains_key(&id) {
                            let v = equipped.equipped.get_mut(&id).unwrap();
                            v.insert(entity);
                        } else {
                            equipped.equipped.insert(id, [entity].into());
                        }
                        equipped
                            .equipped
                            .get_mut(&equipment_type.id())
                            .unwrap()
                            .insert(entity);

                        // Add a few more components to the entity that will let it function as equipment
                        // The actual equipment (Like [`Weapon`]), and the overall [`Item`]. There is some redundancy -
                        // this can be refactored somewhat
                        let mut cmd = world.commands();
                        let mut entity = cmd.entity(entity);

                        // Insert the item and some other necessary stuff
                        let item_name = retrieved_item.name.clone();
                        entity.insert((
                            retrieved_item,
                            TransformBundle::default(),
                            Heat::default(),
                            Name::new(item_name),
                            equipment.0,
                        ));

                        match equipment_type {
                            EquipmentType::Weapon(weapon) => entity.insert(weapon.clone()),
                            EquipmentType::RepairBot(repair) => entity.insert(repair.clone()),
                            EquipmentType::Generator(generator) => entity.insert(generator.clone()),
                            EquipmentType::Battery(battery) => entity.insert(battery.clone()),
                            EquipmentType::Armor(armor) => entity.insert(armor.clone()),
                        };
                    } else {
                        let equipment_type = retrieved_item.equipment.unwrap();

                        // Remove the entity in the equipped
                        equipped
                            .equipped
                            .get_mut(&equipment_type.id())
                            .unwrap()
                            .remove(&entity);

                        let mut cmd = world.commands();
                        let mut entity = cmd.entity(entity);

                        entity.remove::<Item>();

                        match equipment_type {
                            EquipmentType::Weapon(_) => entity.remove::<Weapon>(),
                            EquipmentType::RepairBot(_) => entity.remove::<RepairBot>(),
                            EquipmentType::Generator(_) => entity.remove::<Generator>(),
                            EquipmentType::Battery(_) => entity.remove::<Battery>(),
                            EquipmentType::Armor(_) => entity.remove::<Armor>(),
                        };
                    }
                }
            }
        }
        hooks
            .on_add(|mut world, entity, _component_id| {
                modify(&mut world, entity, true);
            })
            .on_remove(|mut world, entity, _component_id| {
                modify(&mut world, entity, false);
            });
    }
}

/// Attached to certain equipment. When overheated, equipment is disabled until fully cooled.
/// Heat is not passively cooled and will continue to stay at its current score unless modified.
#[derive(Default, Component, Reflect)]
pub struct Heat(f32);

impl Heat {
    /// Get heat value
    pub fn get(&self) -> f32 {
        self.0
    }
}

impl From<f32> for Heat {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Add<f32> for Heat {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Heat((self.0 + rhs).clamp(0f32, 1f32))
    }
}

impl AddAssign<f32> for Heat {
    fn add_assign(&mut self, rhs: f32) {
        self.0 = (self.0 + rhs).clamp(0f32, 1f32);
    }
}

/// Marker struct that disables equipment and is removed when [`Heat`] is 0.
#[derive(Component, Reflect)]
pub struct Overheated;
