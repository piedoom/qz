use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (manage_weapons, manage_projectile_collisions, manage_damage),
        );
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
                                *alliegance,
                                LockedAxes::new().lock_translation_z(),
                                // It might be tempting to store information in the memberships, but this has unintended interactions
                                // Use a component instead for that
                                CollisionLayers {
                                    memberships: PhysicsCategory::Weapon.into(),
                                    filters: LayerMask::from([
                                        PhysicsCategory::Craft,
                                        PhysicsCategory::Structure,
                                    ]),
                                },
                                *transform,
                                RigidBody::Dynamic,
                                // TODO: Cannot have a sensor with density which leads to errors
                                // We can use child colliders for this:
                                // https://github.com/Jondolf/avian/issues/193#issuecomment-1774064306
                                Sensor,
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
    mut collided: Query<(&mut Damage, Option<&Alliegance>), Without<Destroyed>>,
    projectile_hits: Query<(Entity, &CollidingEntities, &Projectile, Option<&Alliegance>)>,
) {
    for (projectile_entity, colliding_entities, projectile, maybe_alliegance) in
        projectile_hits.iter()
    {
        for colliding_entity in colliding_entities.iter() {
            // deteremine if the collided entity is an enemy
            let alliegance = match maybe_alliegance {
                Some(alliegance) => *alliegance,
                None => Alliegance {
                    faction: Faction::empty(),
                    allies: Faction::empty(),
                    enemies: Faction::all(),
                },
            };

            if let Ok((mut damage, maybe_collided_alliegance)) = collided.get_mut(*colliding_entity)
            {
                let other_alliegance = match maybe_collided_alliegance {
                    Some(alliegance) => *alliegance,
                    None => Alliegance {
                        faction: Faction::all(),
                        allies: Faction::all(),
                        enemies: Faction::all(),
                    },
                };
                if alliegance.enemies.contains(other_alliegance.faction) {
                    **damage += projectile.damage as f32;
                    cmd.entity(projectile_entity).despawn_recursive();
                }
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
