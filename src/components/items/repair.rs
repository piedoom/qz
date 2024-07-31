use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Clone, Debug, Component, Reflect, Serialize, Deserialize)]
pub struct RepairBot {
    /// Repair rate / s
    pub rate: f32,
}

#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct Armor {
    /// Armor amount
    pub health: usize,
}

impl Component for Armor {
    const STORAGE_TYPE: bevy::ecs::component::StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks
            .on_add(|mut world, entity, _component_id| {
                if let (Some(parent), Some(armor)) =
                    (world.get::<Parent>(entity), world.get::<Self>(entity))
                {
                    let armor_health = armor.health;
                    if let Some(mut health) = world.get_mut::<Health>(parent.get()) {
                        health.0 += armor_health
                    }
                }
            })
            .on_remove(|mut world, entity, _component_id| {
                if let (Some(parent), Some(armor)) =
                    (world.get::<Parent>(entity), world.get::<Self>(entity))
                {
                    let armor_health = armor.health;
                    if let Some(mut health) = world.get_mut::<Health>(parent.get()) {
                        health.0 -= armor_health
                    }
                }
            });
    }
}
