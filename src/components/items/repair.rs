use bevy::{
    ecs::{
        component::{ComponentHooks, StorageType},
        world::DeferredWorld,
    },
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Repairs damage at a specified rate
#[derive(Clone, Debug, Default, Component, Reflect, Serialize, Deserialize, PartialEq)]
pub struct RepairBot {
    /// Repair rate / s
    pub rate: f32,
}

/// Increases overal [`Health`]
#[derive(Clone, Debug, Default, Reflect, Serialize, Deserialize, PartialEq)]
pub struct Armor {
    /// Armor amount
    pub health: usize,
}

impl Component for Armor {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut ComponentHooks) {
        /// Helper function since add/remove are the same except for the operation
        fn modify_health(world: &mut DeferredWorld, entity: Entity, add: bool) {
            if let (Some(parent), Some(armor)) =
                (world.get::<Parent>(entity), world.get::<Armor>(entity))
            {
                let armor_health = armor.health;
                if let Some(mut health) = world.get_mut::<Health>(parent.get()) {
                    if add {
                        health.add_bonus(armor_health);
                    } else {
                        health.remove_bonus(armor_health);
                    }
                }
            }
        }
        hooks
            .on_add(|mut world, entity, _component_id| {
                modify_health(&mut world, entity, true);
            })
            .on_remove(|mut world, entity, _component_id| {
                modify_health(&mut world, entity, false);
            });
    }
}
