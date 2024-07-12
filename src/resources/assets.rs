use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};

use super::Settings;

/// Assets loaded by [`bevy_asset_loader`]
#[derive(AssetCollection, Resource)]
pub struct Library {
    /// [`Settings`] `.ron` file
    #[asset(key = "settings")]
    pub settings: Handle<Settings>,
    #[asset(key = "items", collection(typed, mapped))]
    pub items: HashMap<String, Handle<Item>>,
    #[asset(key = "creatures", collection(typed, mapped))]
    pub creatures: HashMap<String, Handle<Creature>>,
    #[asset(key = "crafts", collection(typed, mapped))]
    pub crafts: HashMap<String, Handle<Craft>>,
}

/// Creatures are never instantiated, they are constructed via systems
#[derive(Debug, Clone, Component, Reflect, Asset, Serialize, Deserialize)]
pub struct Creature {
    pub name: String,
    pub craft: String,
    pub drops: Vec<(String, DropRate)>,
    pub inventory: Vec<(String, usize)>,
    pub equipped: Vec<(String, usize)>,
    pub range: f32,
}
