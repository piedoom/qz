use avian3d::collision::{Collider, ScalableCollider};
use bevy::prelude::*;

use crate::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (draw_controllers, draw_projectiles, draw_health_and_damage),
        );
    }
}

fn draw_controllers(
    mut gizmos: Gizmos,
    controllers: Query<(Entity, &Transform), With<Controller>>,
    players: Query<&Player>,
    destroyed: Query<&Destroyed>,
) {
    for (entity, transform) in controllers.iter() {
        let pos = transform.translation;
        let f = transform.forward();
        let mut color = match players.get(entity).is_ok() {
            true => Color::srgb(0., 1., 0.),
            false => Color::WHITE,
        };
        if destroyed.get(entity).is_ok() {
            color = Color::srgb(1., 0., 0.);
        }
        gizmos.cuboid(*transform, color);
        gizmos.arrow(pos - *f, pos + *f, color);
    }
}

fn draw_projectiles(
    mut gizmos: Gizmos,
    projectiles: Query<(&Transform, &Collider), With<Projectile>>,
) {
    for (transform, collider) in projectiles.iter() {
        gizmos.sphere(
            transform.translation,
            default(),
            collider.shape().as_ball().unwrap().radius,
            Color::srgb(1.0, 0.0, 0.0),
        );
    }
}

fn draw_health_and_damage(
    mut gizmos: Gizmos,
    health_and_damage: Query<(&Transform, &Health, &Damage), Without<Destroyed>>,
) {
    for (transform, health, damage) in health_and_damage.iter() {
        gizmos.rect_2d(
            transform.translation.truncate(),
            0f32,
            Vec2::new((**health as f32 - **damage as f32) / **health as f32, 0.2),
            Color::srgb(0.0, 1.0, 0.0),
        );
    }
}
