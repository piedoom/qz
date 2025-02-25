use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::prelude::*;

/// Draw debug menus and things in the world
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DrawInspector>()
            .add_plugins((
                WorldInspectorPlugin::new()
                    .run_if(resource_equals::<DrawInspector>(DrawInspector(true))),
                // PhysicsDebugPlugin::default(),
            ))
            .add_systems(
                Update,
                (
                    draw_controllers,
                    draw_projectiles,
                    draw_lasers,
                    draw_health_and_damage,
                    draw_structures,
                    draw_destroyed,
                    draw_chests,
                    draw_active_chests,
                    draw_grids,
                )
                    .run_if(resource_equals::<DrawInspector>(DrawInspector(true))),
            );
    }
}

/// Draw debug grid gizmo
fn draw_grids(mut gizmos: Gizmos) {
    gizmos
        .grid_3d(
            Vec3::new(0f32, 0f32, 0f32),
            default(),
            UVec3::new(128, 128, 0),
            Vec3::splat(16f32),
            Color::srgba(1f32, 1f32, 1f32, 0.02f32),
        )
        .outer_edges();
}

/// System to draw controller entities and their heading
///
/// # System overview
///
/// 1. Get all entities with a [`Collider`]] and [`Controller`]
/// 2. Assume the collider is a ball and get its radius
/// 3. Check if the entity is [`Destroyed`] and adjust the color accordingly
/// 4. Draw debug lines for the controller and its heading
///
/// # Panics
///
/// If the [`Controller`] [`Collider`] is not a ball
fn draw_controllers(
    mut gizmos: Gizmos,
    controllers: Query<(Entity, &Transform, &Collider), With<Controller>>,
    players: Query<&Player>,
    destroyed: Query<&Destroyed>,
) {
    for (entity, transform, collider) in controllers.iter() {
        let size = collider.shape().as_ball().unwrap().radius * 2f32;
        let pos = transform.translation;
        let f = transform.forward();
        let mut color = match players.get(entity).is_ok() {
            true => Color::srgb(0., 1., 0.),
            false => Color::WHITE,
        };
        if destroyed.get(entity).is_ok() {
            color = Color::srgb(1., 0., 0.);
        }
        gizmos.cuboid(transform.with_scale(Vec3::splat(size)), color);
        gizmos.arrow(pos - *f, pos + *f, color);
    }
}

/// System to draw projectiles
///
/// # System overview
///
/// 1. Get all entities with a [`Collider`] and a [`Projectile`]
/// 2. Assume the projectile is a ball and get its radius
/// 3. Draw debug a sphere with the projectile radius
///
/// # Panics
///
/// If the projectile collider is not a ball
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

/// System to draw lasers
///
/// # System overview
///
/// 1. Get all entities with a [`Collider`] and a [`Projectile`]
/// 2. Assume the projectile is a ball and get its radius
/// 3. Draw debug a sphere with the projectile radius
fn draw_lasers(mut gizmos: Gizmos, weapons: Query<(&Weapon, &GlobalTransform)>) {
    for (weapon, transform) in weapons
        .iter()
        .filter(|(w, _)| w.wants_to_fire && matches!(w.weapon_type, WeaponType::LaserWeapon { .. }))
    {
        if let WeaponType::LaserWeapon { range, width, .. } = weapon.weapon_type {
            let transform = transform.compute_transform();
            let mut rotation = transform;
            rotation.rotate_local_y(90f32.to_radians());

            gizmos.rect(
                transform.translation + (transform.down() * range * 0.5),
                rotation.rotation,
                Vec2::new(width, range),
                Color::srgb(1.0, 0.0, 0.0),
            )
        } else {
            unreachable!()
        };
    }
}

/// System to draw health bars on entities with health and damage
///
/// # System overview
///
/// 1. Get all entities with [`Health`] and [`Damage`] components that are not [`Destroyed`]
/// 2. Find the normalized health value and draw a rectangle with that value scaling the width
fn draw_health_and_damage(
    mut gizmos: Gizmos,
    health_and_damage: Query<(&Transform, &Health, &Damage), Without<Destroyed>>,
) {
    for (transform, health, damage) in health_and_damage.iter() {
        gizmos.rect(
            transform.translation,
            default(),
            Vec2::new((health.get() as f32 - **damage) / health.get() as f32, 0.2),
            Color::srgb(0.0, 1.0, 0.0),
        );
    }
}

/// System to draw structures
///
/// # System overview
///
/// 1. Get all entities marked as a [`Structure`] and retrieve optional components [`Spawner`] and[`Destroyed`]
/// 2. Set the color based on the structure type
/// 3. Assume the collider is a ball and get the radius
/// 4. Draw debug with the collider size
///
/// # Panics
///
/// If the collider is not a ball
fn draw_structures(
    mut gizmos: Gizmos,
    structures: Query<
        (&Transform, &Collider, Option<&Destroyed>, Option<&Spawner>),
        With<Structure>,
    >,
) {
    for (transform, collider, maybe_destroyed, maybe_spawner) in structures.iter() {
        let color = if maybe_destroyed.is_none() {
            match maybe_spawner.is_some() {
                true => Color::srgb(0.9, 0.7, 0.1),
                false => Color::srgb(0.4, 0., 1.),
            }
        } else {
            Color::srgb(1., 0., 0.)
        };
        gizmos.cuboid(
            transform.with_scale(Vec3::splat(
                collider.shape().as_ball().unwrap().radius * 2.0,
            )),
            color,
        );
    }
}

/// System to draw overlays over destroyed entities
///
/// # System overview
///
/// 1. Get all entities marked as [`Destroyed`]
/// 2. Draw an "X" over each entity
fn draw_destroyed(mut gizmos: Gizmos, destroyed: Query<&Transform, With<Destroyed>>) {
    /// Red color for destroyed
    const COLOR: Color = Color::srgb(1., 0., 0.);
    for transform in destroyed.iter() {
        let rect = Rect::from_center_size(transform.translation.truncate(), Vec2::splat(1.5));
        let z = transform.translation.z;
        gizmos.line(rect.min.extend(z), rect.max.extend(z), COLOR);
        gizmos.line(
            (rect.min.x, rect.max.y, z).into(),
            (rect.max.x, rect.min.y, z).into(),
            COLOR,
        );
    }
}

/// System to draw chests
///
/// # System overview
///
/// 1. Get all entities marked as a [`Chest`] and get their [`Inventory`]
/// 2. Draw the chest, and a sphere inside the chest if it's inventory isn't empty
/// 3. Get all entities marked as a [`Chest`] that also have [`Credits`] attached
/// 4. Draw the chest as a credits chest
fn draw_chests(
    mut gizmos: Gizmos,
    chests: Query<(&Transform, &Inventory), (With<Chest>, Without<Credits>)>,
    credits: Query<&Transform, (With<Chest>, With<Credits>, Without<Inventory>)>,
) {
    /// Color constant
    const COLOR: Color = Color::srgb(1., 0., 1.);
    for (transform, inventory) in chests.iter() {
        gizmos.cuboid(*transform, COLOR);
        if !inventory.is_empty() {
            gizmos.sphere(transform.translation, default(), 0.5, COLOR);
        }
    }

    for transform in credits.iter() {
        gizmos.sphere(
            transform.translation,
            transform.rotation,
            0.3f32,
            Color::srgb(1., 1., 0.),
        );
    }
}

/// Draw chests within range of an entity that can pick it up
///
/// # System overview
///
/// 1. Get all entities in every [`ChestsInRange`] component
/// 2. Get the [`Transform`] for each chest entity
/// 3. Draw a cirlce around the chest
fn draw_active_chests(
    mut gizmos: Gizmos,
    chests_in_range: Query<&ChestsInRange>,
    transforms: Query<&Transform>,
) {
    /// Purple
    const COLOR: Color = Color::srgb(1., 0., 1.);
    for chests in chests_in_range.iter() {
        for chest in chests.chests.iter() {
            if let Ok(transform) = transforms.get(*chest) {
                gizmos.circle(transform.translation, Dir3::Z, 1f32, COLOR);
            }
        }
    }
}
