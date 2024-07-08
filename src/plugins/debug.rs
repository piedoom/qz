use avian3d::collision::Collider;
use bevy::prelude::*;

use crate::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_controllers,
                draw_projectiles,
                draw_health_and_damage,
                draw_reference_grid,
                draw_structures,
            ),
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

fn draw_reference_grid(mut gizmos: Gizmos) {
    // get 2d point
    gizmos.grid_2d(
        Vec2::ZERO,
        0f32,
        UVec2::new(64, 64),
        Vec2::splat(8f32),
        Color::srgba(1f32, 1f32, 1f32, 0.05f32),
    );
    gizmos
        .grid_3d(
            Vec3::new(0f32, 0f32, -64f32),
            default(),
            UVec3::new(128, 128, 0),
            Vec3::splat(8f32),
            Color::srgba(1f32, 1f32, 1f32, 0.01f32),
        )
        .outer_edges();
    gizmos
        .grid_3d(
            Vec3::new(0f32, 0f32, -128f32),
            default(),
            UVec3::new(256, 256, 0),
            Vec3::splat(8f32),
            Color::srgba(1f32, 1f32, 1f32, 0.01f32),
        )
        .outer_edges();
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
            Vec2::new((**health as f32 - **damage) / **health as f32, 0.2),
            Color::srgb(0.0, 1.0, 0.0),
        );
    }
}

fn draw_structures(mut gizmos: Gizmos, structures: Query<(&Transform, &Structure)>) {
    for (transform, _structure) in structures.iter() {
        gizmos.cuboid(*transform, Color::srgb(0.4, 0., 1.));
    }
}
