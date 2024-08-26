use crate::prelude::*;
use bevy::prelude::*;

pub struct UtilityPlugin;

impl Plugin for UtilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (manage_lifetimes, follow_camera, manage_distance_lifetimes)
                .run_if(in_state(AppState::main())),
        );
    }
}

fn manage_lifetimes(mut cmd: Commands, lifetimes: Query<(Entity, &Lifetime)>, time: Res<Time>) {
    for (entity, lifetime) in lifetimes.iter() {
        if time.elapsed() >= lifetime.created + lifetime.lifetime {
            cmd.entity(entity).despawn_recursive();
        }
    }
}

fn manage_distance_lifetimes(
    mut cmd: Commands,
    distance_lifetimes: Query<(Entity, &Transform, &DistanceLifetime)>,
) {
    for (entity, transform, distance_lifetime) in distance_lifetimes.iter() {
        let distance_squared = distance_lifetime
            .created
            .distance_squared(transform.translation);
        if distance_squared >= distance_lifetime.length.powi(2) {
            cmd.entity(entity).despawn_recursive();
        }
    }
}

fn follow_camera(
    mut camera_transform: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_transform: Query<&Transform, With<Player>>,
) {
    if let (Ok(mut camera_transform), Ok(player_transform)) = (
        camera_transform.get_single_mut(),
        player_transform.get_single(),
    ) {
        let difference = player_transform.translation - camera_transform.translation;
        camera_transform.translation += difference.truncate().extend(0f32);
    }
}
