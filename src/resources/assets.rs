use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;

use super::Settings;

/// Assets loaded by [`bevy_asset_loader`]
#[derive(AssetCollection, Resource)]
pub struct Library {
    /// [`Settings`] `.ron` file
    #[asset(key = "settings")]
    pub settings: Handle<Settings>,
    #[asset(key = "items", collection(typed, mapped))]
    pub items: HashMap<String, Handle<Item>>,
}
