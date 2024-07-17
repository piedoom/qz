mod ai;
mod assets;
mod controllers;
mod debug;
mod equipment;
mod input;
mod inventory;
mod settings;
mod structures;
mod ui;
mod utility;
mod weapons;
mod world;

use avian3d::{prelude::Gravity, PhysicsPlugins};
use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_egui::EguiPlugin;

use crate::prelude::AppState;

/// Plugins required for displaying the game on a client device
pub struct ClientPlugins;

/// Initialize necessary client components
struct ClientInitPlugin;

impl PluginGroup for ClientPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add_group(DefaultPlugins)
            .add_group(PhysicsPlugins::default())
            .add(EguiPlugin)
            .add(ClientInitPlugin)
            .add(assets::AssetsPlugin)
            .add(settings::SettingsPlugin)
            .add(input::InputPlugin)
            .add(debug::DebugPlugin)
            .add(world::WorldPlugin)
            .add(controllers::ControllersPlugin)
            .add(ai::AiPlugin)
            .add(inventory::InventoryPlugin)
            .add(weapons::WeaponsPlugin)
            .add(utility::UtilityPlugin)
            .add(ui::UiPlugin)
            .add(equipment::RepairsPlugin)
            .add(structures::StructuresPlugin)
    }
}

/// Initialize anything else needed for the client
impl Plugin for ClientInitPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .insert_resource(Gravity(Vec3::ZERO));
    }
}
