mod ai;
mod assets;
mod controllers;
mod debug;
mod input;
mod settings;
mod world;

use avian3d::{prelude::Gravity, PhysicsPlugins};
use bevy::{app::PluginGroupBuilder, math::VectorSpace, prelude::*};

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
            .add(ClientInitPlugin)
            .add(assets::AssetsPlugin)
            .add(settings::SettingsPlugin)
            .add(input::InputPlugin)
            .add(debug::DebugPlugin)
            .add(world::WorldPlugin)
            .add(controllers::ControllersPlugin)
            .add(ai::AiPlugin)
    }
}

/// Initialize anything else needed for the client
impl Plugin for ClientInitPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .insert_resource(Gravity(Vec3::ZERO));
    }
}
