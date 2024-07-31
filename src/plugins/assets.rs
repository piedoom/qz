use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use crate::prelude::*;

/// Loads all assets
pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<Settings>::new(&["settings.ron"]))
            .add_plugins(RonAssetPlugin::<Item>::new(&[
                "item.ron",
                "weapon.ron",
                "repair.ron",
                "generator.ron",
                "battery.ron",
                "armor.ron",
            ]))
            .add_plugins(RonAssetPlugin::<Creature>::new(&["creature.ron"]))
            .add_plugins(RonAssetPlugin::<Craft>::new(&["craft.ron"]))
            .add_plugins(RonAssetPlugin::<Building>::new(&["building.ron"]))
            // Continue to the main game state once everything is loaded in, so
            // we can be sure all assets are loaded first
            .add_loading_state(
                LoadingState::new(AppState::preloading())
                    .continue_to_state(AppState::main())
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "default.assets.ron",
                    )
                    .load_collection::<Library>(),
            );
    }
}
