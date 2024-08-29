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

fn default_margin() -> Option<f32> {
    Some(0.3f32)
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
    #[serde(default = "usize::max_value")]
    pub inventory_space: usize,
    /// Equipped items on spawn
    #[serde(default)]
    pub equipped: EquippedBuilder,
    /// If spawned from a spawner, denote that here
    #[serde(default)]
    pub spawner: Option<Spawner>,
    /// If a store, potential items and their chance of being available
    #[serde(default)]
    pub store: Option<Vec<(String, Chance)>>,
    /// Percentage amount to deduct when buying
    #[serde(default = "default_margin")]
    pub store_margin: Option<f32>,
    /// Starting credits, if any
    #[serde(default)]
    pub credits: Option<usize>,
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

pub struct Zone {
    /// The name identifier of this zone
    pub name: String,
    /// The depth of this `Zone` in the [`Universe`]. This can help dictate difficulty
    pub depth: usize,
    pub scene: Option<Handle<DynamicScene>>,
}

impl Zone {
    pub fn from_depth(depth: usize) -> Self {
        Self {
            name: (0..2)
                .map(|_| random_word::gen(random_word::Lang::En).to_string())
                .reduce(|acc, e| acc + " " + &e)
                .unwrap(),
            depth,
            scene: None,
        }
    }
}

impl Library {
    pub fn creature(&self, name: impl AsRef<str>) -> Option<Handle<Creature>> {
        self.creatures
            .get(&format!("creatures/{}.creature.ron", name.as_ref()))
            .cloned()
    }

    pub fn craft(&self, name: impl AsRef<str>) -> Option<Handle<Craft>> {
        self.crafts
            .get(&format!("crafts/{}.craft.ron", name.as_ref()))
            .cloned()
    }

    pub fn building(&self, name: impl AsRef<str>) -> Option<Handle<Building>> {
        self.buildings
            .get(&format!("buildings/{}.building.ron", name.as_ref()))
            .cloned()
    }

    pub fn item(&self, name: impl AsRef<str>) -> Option<Handle<Item>> {
        self.items
            .get(&format!("items/{}.ron", name.as_ref()))
            .cloned()
    }

    pub fn model(&self, name: impl AsRef<str>) -> Option<Handle<Scene>> {
        // e.g., name is category_name/model_name. Convert to models/category_name/model_name/model_name.gltf
        let (category_name, model_name) = name.as_ref().split_once('/').unwrap();

        self.models
            .get(&format!(
                "models/{0}/{1}/{1}.gltf",
                category_name, model_name
            ))
            .cloned()
    }
}
