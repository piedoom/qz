use bevy::{
    ecs::{
        component::{ComponentHooks, StorageType},
        world::DeferredWorld,
    },
    prelude::*,
    utils::{hashbrown::HashMap, HashSet},
};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Equipment needs to use the parent/child tree. This allows
/// for multiple equips of the same type to be used at once
#[derive(Debug, Default, Component, Clone, Reflect, Serialize, Deserialize)]
pub struct Equipped {
    /// A map of [`EquipmentType`]s to equipped entities. This is updated via hooks and is hopefully never invalid.
    /// This value is essentially a cache.
    pub equipped: HashMap<EquipmentTypeId, HashSet<Entity>>,
    pub slots: HashMap<EquipmentTypeId, usize>,
    // pub inventory: Inventory,
}

impl Equipped {
    pub fn available(&self, equipment_type: &EquipmentTypeId) -> usize {
        let max = self.slots.get(equipment_type).cloned().unwrap_or_default();
        let current = self
            .equipped
            .get(equipment_type)
            .map(|x| x.len())
            .unwrap_or_default();
        max - current
    }

    pub fn mass(&self, item_assets: &Assets<Item>, items: &Query<&Equipment>) -> f32 {
        self.equipped
            .iter()
            .map(|(_, e)| e.iter())
            .flatten()
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
pub struct EquippedBuilder {
    pub equipped: Vec<String>,
    pub slots: Vec<(EquipmentTypeId, usize)>,
}

/// Allows us to specify specific equipment categories
#[derive(Debug, Reflect, Clone, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon(Weapon),
    RepairBot(RepairBot),
    Generator(Generator),
    Battery(Battery),
    Armor(Armor),
}

#[derive(Copy, Clone, Debug, Reflect, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EquipmentTypeId {
    Weapon,
    RepairBot,
    Generator,
    Battery,
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
    pub fn handle(&self) -> Handle<Item> {
        self.0.clone()
    }

    pub fn new(item_handle: Handle<Item>) -> Self {
        Self(item_handle)
    }
}

impl Component for Equipment {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    // When adding an item as a component...
    fn register_component_hooks(hooks: &mut ComponentHooks) {
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

                        dbg!(&equipped);
                        dbg!(&equipment_type.id());

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

                        // Insert the item
                        entity.insert(retrieved_item);

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
