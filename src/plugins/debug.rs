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
                draw_destroyed,
                draw_chests,
                draw_active_chests,
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
            true => {
                gizmos.line(pos, pos + (*f * 16f32), Color::srgba(1., 1., 1., 0.1));
                Color::srgb(0., 1., 0.)
            }
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

fn draw_structures(
    mut gizmos: Gizmos,
    structures: Query<(&Transform, &Structure, Option<&Destroyed>, Option<&Spawner>)>,
) {
    for (transform, _structure, maybe_destroyed, maybe_spawner) in structures.iter() {
        let color = if maybe_destroyed.is_none() {
            match maybe_spawner.is_some() {
                true => Color::srgb(0.9, 0.7, 0.1),
                false => Color::srgb(0.4, 0., 1.),
            }
        } else {
            Color::srgb(1., 0., 0.)
        };
        gizmos.cuboid(*transform, color);
    }
}

fn draw_destroyed(mut gizmos: Gizmos, destroyed: Query<&Transform, With<Destroyed>>) {
    const COLOR: Color = Color::srgb(1., 0., 0.);
    for transform in destroyed.iter() {
        let rect = Rect::from_center_size(transform.translation.truncate(), Vec2::splat(1.5));
        gizmos.line_2d(rect.min, rect.max, COLOR);
        gizmos.line_2d(
            (rect.min.x, rect.max.y).into(),
            (rect.max.x, rect.min.y).into(),
            COLOR,
        );
    }
}

fn draw_chests(mut gizmos: Gizmos, chests: Query<(&Transform, &Inventory), With<Chest>>) {
    const COLOR: Color = Color::srgb(1., 0., 1.);
    for (transform, inventory) in chests.iter() {
        gizmos.cuboid(*transform, COLOR);
        if !inventory.is_empty() {
            gizmos.sphere(transform.translation, default(), 0.5, COLOR);
        }
    }
}

fn draw_active_chests(
    mut gizmos: Gizmos,
    chests_in_range: Query<&ChestsInRange>,
    transforms: Query<&Transform>,
) {
    const COLOR: Color = Color::srgb(1., 0., 1.);
    for chests in chests_in_range.iter() {
        for chest in chests.chests.iter() {
            let transform = transforms.get(*chest).unwrap();
            gizmos.circle(transform.translation, Dir3::Z, 1f32, COLOR);
        }
    }
}
