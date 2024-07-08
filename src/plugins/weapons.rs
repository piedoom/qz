use crate::prelude::*;
use avian3d::{
    collision::{Collider, CollidingEntities, CollisionLayers},
    prelude::{LinearVelocity, LockedAxes, RigidBody},
};
use bevy::prelude::*;
use rand::Rng;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                manage_weapons,
                set_craft_collison_layers,
                manage_projectile_collisions,
                manage_damage,
            ),
        );
    }
}

/// Set collision layers based off of alliegances for crafts specifically
fn set_craft_collison_layers(
    mut cmd: Commands,
    alliegances: Query<(Entity, &Alliegance), (Changed<Alliegance>, With<Craft>)>,
) {
    for (entity, alliegance) in alliegances.iter() {
        cmd.entity(entity).insert(CollisionLayers {
            memberships: alliegance.allies.into(),
            filters: alliegance.enemies.union(alliegance.allies).into(),
        });
    }
}

/// Fire weapons when appropriate
fn manage_weapons(
    mut cmd: Commands,
    mut weapons: Query<(&Parent, &mut Weapon)>,
    destroyed: Query<&Destroyed>,
    transforms: Query<&Transform>,
    alliegances: Query<&Alliegance>,
    linear_velocity: Query<&LinearVelocity>,
    time: Res<Time>,
) {
    for (parent, mut weapon) in weapons.iter_mut() {
        // Skip destroyed entities
        if destroyed.get(parent.get()).is_ok() {
            continue;
        }
        let linear_velocity = linear_velocity.get(parent.get()).unwrap();
        let transform = transforms.get(parent.get()).unwrap();
        let alliegance = alliegances.get(parent.get()).unwrap();
        if weapon.wants_to_fire {
            match weapon.weapon_type {
                WeaponType::Projectile {
                    speed,
                    recoil,
                    damage,
                    radius,
                    spread,
                    shots,
                    lifetime,
                } => {
                    // Check if weapon can fire
                    if weapon.last_fired + recoil <= time.elapsed() {
                        for _ in 0..shots {
                            let mut spread_angle = 0f32;
                            if spread != 0f32 {
                                let half_spread = spread / 2f32;
                                spread_angle =
                                    rand::thread_rng().gen_range(-half_spread..half_spread);
                            }
                            // Spawn a projectile
                            cmd.spawn((
                                Projectile { damage },
                                LockedAxes::new().lock_translation_z(),
                                CollisionLayers::new(
                                    alliegance.allies.bits(),
                                    alliegance.enemies.bits(),
                                ),
                                *transform,
                                RigidBody::Dynamic,
                                Collider::sphere(radius),
                                LinearVelocity(
                                    transform
                                        .rotation
                                        .mul_quat(Quat::from_rotation_y(spread_angle))
                                        .mul_vec3(-Vec3::Z * speed)
                                        + linear_velocity.0,
                                ),
                                Lifetime {
                                    created: time.elapsed(),
                                    lifetime,
                                },
                            ));
                        }
                        // Set the last fired time and set "wants to fire" to false
                        weapon.last_fired = time.elapsed();
                        weapon.wants_to_fire = false;
                    }
                }
            }
        }
    }
}

fn manage_projectile_collisions(
    mut cmd: Commands,
    mut damage: Query<&mut Damage, Without<Destroyed>>,
    projectile_hits: Query<(Entity, &CollidingEntities, &Projectile)>,
) {
    for (projectile_entity, colliding_entities, projectile) in projectile_hits.iter() {
        for colliding_entity in colliding_entities.iter() {
            if let Ok(mut damage) = damage.get_mut(*colliding_entity) {
                **damage += projectile.damage as f32;
                cmd.entity(projectile_entity).despawn_recursive();
            }
        }
    }
}

fn manage_damage(
    mut cmd: Commands,
    health_and_damage: Query<(Entity, &Health, &Damage), Changed<Damage>>,
) {
    for (entity, health, damage) in health_and_damage.iter() {
        if **damage >= **health as f32 {
            cmd.entity(entity).insert(Destroyed);
        }
    }
}
