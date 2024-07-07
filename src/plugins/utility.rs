use crate::prelude::*;
use bevy::prelude::*;

pub struct UtilityPlugin;

impl Plugin for UtilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (manage_lifetimes,));
    }
}

fn manage_lifetimes(mut cmd: Commands, lifetimes: Query<(Entity, &Lifetime)>, time: Res<Time>) {
    for (entity, lifetime) in lifetimes.iter() {
        if time.elapsed() >= lifetime.created + lifetime.lifetime {
            cmd.entity(entity).despawn_recursive();
        }
    }
}
