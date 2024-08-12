use crate::prelude::*;
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    utils::HashMap,
};
use bevy_asset_loader::prelude::*;
use serde::{Deserialize, Serialize};

use super::Settings;

/// Assets loaded by [`bevy_asset_loader`]
#[derive(AssetCollection, Resource, Clone)]
pub struct Library {
    /// [`Settings`] `.ron` file
    #[asset(key = "settings")]
    pub settings: Handle<Settings>,
    /// All [`Item`]s
    #[asset(key = "items", collection(typed, mapped))]
    pub items: HashMap<String, Handle<Item>>,
    /// All [`Creature`]s
    #[asset(key = "creatures", collection(typed, mapped))]
    pub creatures: HashMap<String, Handle<Creature>>,
    /// All [`Craft`]s
    #[asset(key = "crafts", collection(typed, mapped))]
    pub crafts: HashMap<String, Handle<Craft>>,
    /// All [`Building`]s
    #[asset(key = "buildings", collection(typed, mapped))]
    pub buildings: HashMap<String, Handle<Building>>,
    /// All GLTF scenes as models
    #[asset(key = "models", collection(typed, mapped))]
    pub models: HashMap<String, Handle<Scene>>,
}

/// Creatures are never instantiated, they are constructed via systems
#[derive(Debug, Clone, Component, Reflect, Asset, Serialize, Deserialize)]
pub struct Creature {
    /// Creature name string
    pub name: String,
    /// Craft this creature will use
    pub craft: String,
    /// Model this creature will use
    pub model: String,
    /// What this creature will drop
    #[serde(default)]
    pub drops: Vec<(String, DropRate)>,
    /// Creature inventory
    #[serde(default)]
    pub inventory: Vec<(String, usize)>,
    /// Cretaure's equipped items
    pub equipped: EquippedBuilder,
    /// Sight range of this creature
    pub range: f32,
    /// Range of credits for this creature, from a minimum to maximum limit
    #[serde(default)]
    pub credits: (usize, usize),
}

/// Buildings are never instantiated, they are constructed via systems
#[derive(Debug, Clone, Component, Reflect, Asset, Serialize, Deserialize)]
pub struct Building {
    /// Building name string
    pub name: String,
    /// Mass of the building
    pub mass: f32,
    /// Health of the building
    pub health: usize,
    /// Size of the building
    pub size: f32,

    /// What this building drops on destruction
    #[serde(default)]
    pub drops: Vec<(String, DropRate)>,

    /// Items within this building's inventory
    #[serde(default)]
    pub inventory: Vec<(String, usize)>,
    /// Inventory capacity
    pub inventory_space: usize,
    /// Equipped items on spawn
    #[serde(default)]
    pub equipped: EquippedBuilder,
    /// If spawned from a spawner, denote that here
    #[serde(default)]
    pub spawner: Option<Spawner>,
    /// If a store, denote that here along with sale options
    #[serde(default)]
    pub store: Option<Vec<(String, SaleOptions)>>,
    /// Starting credits, if any
    #[serde(default)]
    pub credits: Option<usize>,
}

/// A serializable setup for the zone that will be spawned
#[derive(Resource, Default, Asset, Reflect, Serialize, Deserialize)]
pub struct ZoneDescription {
    /// Buildings in this zone
    pub buildings: Vec<trigger::SpawnBuilding>,
    /// Gates in this zone
    pub gates: Vec<trigger::SpawnGate>,
}

/// Background material
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BackgroundMaterial {
    /// Camera position
    #[uniform(101)]
    pub position: Vec2,
}

impl Material for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/background.wgsl".into()
    }
}
