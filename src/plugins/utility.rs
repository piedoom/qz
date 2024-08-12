use crate::prelude::*;
use avian3d::prelude::PhysicsSet;
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
    if let (Ok(mut camera_transform), Ok(player_transform)) = (
        camera_transform.get_single_mut(),
        player_transform.get_single(),
    ) {
        *camera_transform = Transform::from_translation(
            player_transform.translation + Vec3::new(0f32, -6f32, 20.0),
        )
        .looking_at(player_transform.translation, Dir3::Z);
    }
}
