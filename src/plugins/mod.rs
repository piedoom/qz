mod ai;
mod assets;
mod controllers;
mod debug;
mod equipment;
mod input;
mod inventory;
mod settings;
mod state;
mod structures;
mod ui;
mod utility;
mod weapons;
mod world;

use avian3d::{prelude::Gravity, PhysicsPlugins};
use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_turborand::prelude::RngPlugin;

use crate::{prelude::AppState, resources::Factions};

/// Plugins required for displaying the game on a client device
pub struct ClientPlugins;

/// Initialize necessary client components
struct ClientInitPlugin;

impl PluginGroup for ClientPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            // Bevy essential
            .add_group(DefaultPlugins)
            // 3rd party
            .add_group(PhysicsPlugins::default())
            .add(EguiPlugin)
            .add(RngPlugin::default())
            .add(ClientInitPlugin)
            // Crate
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
            .insert_resource(Gravity(Vec3::ZERO))
            .init_resource::<Factions>();
    }
}
