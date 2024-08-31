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
use bevy_htnp::prelude::HtnPlanningPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_turborand::prelude::RngPlugin;

use crate::{
    error::GameError,
    prelude::{AppState, DrawInspector},
    resources::Factions,
};

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
            .add(HtnPlanningPlugin::new())
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
            .add(equipment::EquipmentPlugin)
            .add(structures::StructuresPlugin)
            .add(state::StatePlugin)
    }
}

/// Initialize anything else needed for the client
impl Plugin for ClientInitPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .insert_resource(Gravity(Vec3::ZERO))
            .insert_resource(bevy_etcetera::Directories::new("org", "doomy", "qz"))
            .init_resource::<Factions>()
            .add_event::<GameError>();
    }
}
