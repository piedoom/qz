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
    #[asset(key = "buildings", collection(typed, mapped))]
    pub buildings: HashMap<String, Handle<Building>>,
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

/// Buildings are never instantiated, they are constructed via systems
#[derive(Debug, Clone, Component, Reflect, Asset, Serialize, Deserialize)]
pub struct Building {
    pub name: String,
    pub mass: f32,
    pub health: usize,
    pub size: f32,

    #[serde(default)]
    pub drops: Vec<(String, DropRate)>,
    #[serde(default)]
    pub inventory: Vec<(String, usize)>,
    pub inventory_space: usize,
    #[serde(default)]
    pub equipped: Vec<(String, usize)>,

    #[serde(default)]
    pub spawner: Option<Spawner>,
    #[serde(default)]
    pub store: Option<Vec<(String, SaleOptions)>>,
    #[serde(default)]
    pub credits: Option<usize>,
}

#[derive(Serialize, Deserialize, Reflect, Asset, Clone, Debug)]
pub struct WorldRepresentation {
    players: Vec<savefile::PlayerRepresentation>,
}

pub mod savefile {
    use super::*;

    #[derive(Serialize, Deserialize, Reflect, Clone, Debug)]
    pub struct PlayerRepresentation {
        pub player: Player,
        pub inventory: Vec<(String, usize)>,
        pub equipped: Vec<(String, usize)>,
        pub credits: Credits,
        pub slice: Slice,
    }
}
