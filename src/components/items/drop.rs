use std::process::Output;

use bevy::{ecs::component::StorageType, prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Items to drop upon destruction
#[derive(Debug, Clone, Component, Reflect, Deref, DerefMut)]
pub struct Drops(
    /// Items to drop mapped to a range of amount to drop normalized value determining drop rate
    pub HashMap<Handle<Item>, DropRate>,
);

#[derive(Debug, Clone, Reflect, Deref, DerefMut, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct DropsBuilder(pub Vec<(String, DropRate)>);

impl Component for DropsBuilder {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(Self::on_add);
    }
}

impl Builder for DropsBuilder {
    type Output = Drops;

    fn from_output(output: Self::Output) -> Self {
        DropsBuilder(
            output
                .0
                .iter()
                .map(|(handle, count)| {
                    let path = handle.path().unwrap();
                    let name = path
                        .path()
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    (name, *count)
                })
                .collect(),
        )
    }

    fn into_output(builder: Self, library: &Library) -> Self::Output {
        Drops(
            builder
                .0
                .into_iter()
                .map(|(name, drop_rate)| (library.item(name).unwrap(), drop_rate))
                .collect(),
        )
    }
}

/// Describes the chance that certain items will be dropped
#[derive(Debug, Clone, Reflect, PartialEq, Eq, Hash, Deserialize, Serialize, Copy)]
pub struct DropRate {
    /// The minimum amount of items that can drop
    pub min: usize,
    /// The maximum amount of items that can drop
    pub max: usize,
    /// 1 in X chance to drop. 1 will always drop.
    pub d: usize,
}
