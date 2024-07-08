use crate::prelude::*;
use bevy::prelude::*;

pub struct UtilityPlugin;

impl Plugin for UtilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (manage_lifetimes, follow_camera));
    }
}

fn manage_lifetimes(mut cmd: Commands, lifetimes: Query<(Entity, &Lifetime)>, time: Res<Time>) {
    for (entity, lifetime) in lifetimes.iter() {
        if time.elapsed() >= lifetime.created + lifetime.lifetime {
            cmd.entity(entity).despawn_recursive();
        }
    }
}

fn follow_camera(
    mut camera_transform: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_transform: Query<&Transform, With<Player>>,
) {
    if let Ok(mut camera_transform) = camera_transform.get_single_mut() {
        if let Ok(player_transform) = player_transform.get_single() {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        }
    }
}
