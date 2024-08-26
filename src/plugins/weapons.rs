use std::time::Duration;

use crate::prelude::*;
use avian3d::prelude::*;
use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use rand::Rng;

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                manage_weapons,
                manage_projectile_collisions,
                manage_damage,
                manage_laser_collisions,
            )
                .run_if(in_state(AppState::main())),
        );
    }
}

/// Fire weapons when appropriate
fn manage_weapons(
    mut cmd: Commands,
    mut weapons: Query<(
        &mut Weapon,
        &mut Heat,
        Option<&Overheated>,
        Option<&Children>,
    )>,
    mut parents: Query<
        (Entity, &mut Energy, &Equipped, &LinearVelocity, &Alliegance),
        (Without<Destroyed>, With<Transform>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut transforms: Query<&mut Transform>,
    lasers: Query<Entity, (With<Laser>, With<Transform>)>,
    library: Res<Library>,
    time: Res<Time>,
) {
    for (entity, mut total_energy, equipped, linear_velocity, alliegance) in parents.iter_mut() {
        let transform = *transforms.get(entity).unwrap();
        // Get all entities that are weapons
        if let Some(entities) = equipped.equipped.get(&EquipmentTypeId::Weapon) {
            for weapon_entity in entities {
                // Get the weapon component attached
                if let Ok((mut weapon, mut heat, maybe_overheated, maybe_weapon_children)) =
                    weapons.get_mut(*weapon_entity)
                {
                    // Do stuff depending on the weapon
                    match &weapon.weapon_type {
                        WeaponType::ProjectileWeapon {
                            speed,
                            recoil,
                            damage,
                            radius,
                            spread,
                            shots,
                            lifetime,
                            tracking,
                            energy,
                            projectile_model,
                            distance,
                        } => {
                            if weapon.wants_to_fire {
                                // Check if weapon can fire
                                if weapon.last_fired + Duration::from_secs_f32(*recoil)
                                    <= time.elapsed()
                                    && total_energy.consume(*energy as f32).is_ok()
                                {
                                    for _ in 0..*shots {
                                        let mut spread_angle = 0f32;
                                        if spread != &0f32 {
                                            let half_spread = spread / 2f32;
                                            spread_angle = rand::thread_rng()
                                                .gen_range(-half_spread..half_spread);
                                        }

                                        // If we have tracking, find the additional angle
                                        let (direction, added_angle) = {
                                            match weapon.target {
                                                Some(target) => transform
                                                    .calculate_turn_angle(target.truncate()),
                                                None => (RotationDirection::None, 0f32),
                                            }
                                        };

                                        let final_angle =
                                            (-added_angle.min(*tracking).max(-tracking)
                                                * (f32::from(direction)))
                                                + spread_angle;

                                        // Spawn a projectile
                                        cmd.spawn((
                                            LockedAxes::new().lock_translation_z(),
                                            TransformBundle::from_transform(transform),
                                            RigidBody::Dynamic,
                                            Mass(1f32),
                                            LinearVelocity(
                                                transform
                                                    .rotation
                                                    .mul_quat(Quat::from_axis_angle(
                                                        Vec3::Y,
                                                        final_angle,
                                                    ))
                                                    .mul_vec3(-Vec3::Z * *speed)
                                                    + linear_velocity.0,
                                            ),
                                            Lifetime {
                                                created: time.elapsed(),
                                                lifetime: Duration::from_secs_f32(*lifetime),
                                            },
                                            DistanceLifetime {
                                                created: transform.translation,
                                                length: *distance,
                                            },
                                            Projectile { damage: *damage },
                                            alliegance.clone(),
                                            Sensor,
                                            Collider::sphere(*radius),
                                            CollisionLayers {
                                                memberships: PhysicsCategory::Weapon.into(),
                                                filters: LayerMask::from([
                                                    PhysicsCategory::Craft,
                                                    PhysicsCategory::Structure,
                                                ]),
                                            },
                                        ))
                                        .with_children(
                                            |cmd| {
                                                cmd.spawn(SceneBundle {
                                                    scene: library
                                                        .model(projectile_model.clone())
                                                        .unwrap(),
                                                    transform: Transform::default_z()
                                                        .with_scale(Vec3::splat(*radius)),
                                                    ..Default::default()
                                                });
                                            },
                                        );
                                    }
                                    // Set the last fired time and set "wants to fire" to false
                                    weapon.last_fired = time.elapsed();
                                    weapon.wants_to_fire = false;
                                }
                            }
                        }
                        WeaponType::LaserWeapon {
                            tracking,
                            damage_per_second,
                            energy_per_second,
                            range,
                            width,
                            activation_energy,
                            color,
                            heat_per_second,
                            cooling_per_second,
                        } => {
                            let energy_to_consume = energy_per_second * time.delta_seconds();
                            let has_enough_energy = total_energy.charge() >= energy_to_consume;
                            let has_enough_activation_energy =
                                total_energy.charge() >= energy_to_consume + activation_energy;
                            let is_overheated = maybe_overheated.is_some();
                            let wants_to_fire = weapon.wants_to_fire;

                            // Get if any lasers are the child of this weapon already
                            let maybe_lasers = maybe_weapon_children
                                .map(|mwc| mwc.iter().flat_map(|c| lasers.get(*c).ok()));

                            enum WeaponState {
                                // Weapon is inactive, only cool
                                Overheated,
                                // Weapon is already firing. Control lasers
                                Firing,
                                // Weapon is initiating fire
                                StartFiring,
                                // Weapon is not firing
                                Off,
                            }

                            let state = if is_overheated {
                                WeaponState::Overheated
                            } else if wants_to_fire && has_enough_energy && maybe_lasers.is_some() {
                                WeaponState::Firing
                            } else if wants_to_fire && has_enough_activation_energy {
                                WeaponState::StartFiring
                            } else {
                                WeaponState::Off
                            };

                            // Rotate the weapon turret
                            let rot = angle_with_tracking(&weapon, transform, *tracking, 0f32);
                            let mut weapon_transform = transforms.get_mut(*weapon_entity).unwrap();
                            weapon_transform.rotation = rot;

                            match state {
                                WeaponState::Overheated => {
                                    // Despawn child lasers
                                    cmd.entity(*weapon_entity).despawn_descendants();
                                }
                                WeaponState::Firing => {
                                    total_energy
                                        .consume(energy_to_consume)
                                        .expect("should have enough energy");
                                }
                                WeaponState::StartFiring => {
                                    total_energy
                                        .consume(energy_to_consume)
                                        .expect("should have enough energy");
                                    // Create laser
                                    cmd.entity(*weapon_entity).with_children(|cmd| {
                                        cmd.spawn((
                                            Sensor,
                                            Laser {
                                                damage_per_second: *damage_per_second,
                                                range: *range,
                                                width: *width,
                                            },
                                            Collider::cuboid(*width, *width, *range),
                                            // Center
                                            MaterialMeshBundle {
                                                mesh: meshes
                                                    .add(Cuboid::new(*width, *width, *range)),
                                                material: materials.add(StandardMaterial {
                                                    emissive: LinearRgba::rgb(
                                                        color.0, color.1, color.2,
                                                    ),
                                                    ..default()
                                                }),
                                                transform: Transform::from_translation(Vec3::new(
                                                    0f32,
                                                    0f32,
                                                    -range / 2f32,
                                                )),
                                                ..Default::default()
                                            },
                                            NotShadowCaster,
                                            NotShadowReceiver,
                                        ));
                                    });
                                }
                                WeaponState::Off => {
                                    if maybe_lasers.is_some() {
                                        // Remove lasers
                                        cmd.entity(*weapon_entity).despawn_descendants();
                                    }
                                }
                            }

                            // Manage weapon heat
                            *heat += match state {
                                WeaponState::Overheated | WeaponState::Off => {
                                    -cooling_per_second * time.delta_seconds()
                                }
                                WeaponState::StartFiring | WeaponState::Firing => {
                                    heat_per_second * time.delta_seconds()
                                }
                            }
                        }
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
            let alliegance = match maybe_alliegance.cloned() {
                Some(alliegance) => alliegance,
                // If no alliegance, this will damage anything
                None => Alliegance {
                    faction: Faction::default(),
                    allies: FactionSet::default(),
                    enemies: FactionSet::all(),
                },
            };

            // Get collisions
            if let Ok((mut damage, maybe_collided_alliegance)) = collided.get_mut(*colliding_entity)
            {
                // Get the alliegance of the other object
                let other_alliegance = match maybe_collided_alliegance {
                    Some(alliegance) => alliegance,
                    None => &Alliegance::default(),
                };
                if alliegance.enemies.contains(&other_alliegance.faction) {
                    **damage += projectile.damage as f32;
                    cmd.entity(projectile_entity).despawn_recursive();
                }
            }
        }
    }
}

fn manage_laser_collisions(
    mut collided: Query<(&mut Damage, Option<&Alliegance>), Without<Destroyed>>,
    time: Res<Time>,
    laser_hits: Query<(&CollidingEntities, &Laser, Option<&Alliegance>)>,
) {
    for (colliding_entities, laser, maybe_alliegance) in laser_hits.iter() {
        for colliding_entity in colliding_entities.iter() {
            // deteremine if the collided entity is an enemy
            let alliegance = match maybe_alliegance.cloned() {
                Some(alliegance) => alliegance,
                // If no alliegance, this will damage anything
                None => Alliegance {
                    faction: Faction::default(),
                    allies: FactionSet::default(),
                    enemies: FactionSet::all(),
                },
            };

            // Get collisions
            if let Ok((mut damage, maybe_collided_alliegance)) = collided.get_mut(*colliding_entity)
            {
                // Get the alliegance of the other object
                let other_alliegance = match maybe_collided_alliegance {
                    Some(alliegance) => alliegance,
                    None => &Alliegance::default(),
                };
                if alliegance.enemies.contains(&other_alliegance.faction) {
                    **damage += laser.damage_per_second * time.delta_seconds();
                    // cmd.entity(projectile_entity).despawn_recursive();
                }
            }
        }
    }
}

// for hit in sq.shape_hits(
//     &Collider::cuboid(*width, *width, *range),
//     transform.translation + (transform.down() * *range * 0.5),
//     default(),
//     Dir3::new(rot.mul_vec3(transform.forward().into())).unwrap(),
//     *range,
//     128,
//     true,
//     &SpatialQueryFilter {
//         mask: [PhysicsCategory::Craft, PhysicsCategory::Structure]
//             .into(),
//         excluded_entities: [entity].into(),
//     },
// ) {
//     if let Ok(mut damage) = damage.get_mut(hit.entity) {
//         **damage += damage_per_second * time.delta_seconds();
//     }
// }

fn manage_damage(
    mut cmd: Commands,
    health_and_damage: Query<(Entity, &Health, &Damage), Changed<Damage>>,
) {
    for (entity, health, damage) in health_and_damage.iter() {
        if **damage >= health.get() as f32 {
            cmd.entity(entity).insert(Destroyed);
        }
    }
}

/// Utility to find the angle, with tracking in a set range
fn angle_with_tracking(
    weapon: &Weapon,
    transform: Transform,
    tracking: f32,
    spread_angle: f32,
) -> Quat {
    // If we have tracking, find the additional angle
    let (direction, added_angle) = {
        match weapon.target {
            Some(target) => transform.calculate_turn_angle(target.truncate()),
            None => (RotationDirection::None, 0f32),
        }
    };

    Quat::from_axis_angle(
        Vec3::Y,
        (-added_angle.min(tracking).max(-tracking) * (f32::from(direction))) + spread_angle,
    )
}
