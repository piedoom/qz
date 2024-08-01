use std::time::Duration;

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
    mut weapons: Query<&mut Weapon>,
    mut damage: Query<&mut Damage>,
    mut parents: Query<
        (
            Entity,
            &Transform,
            &mut Energy,
            &Equipped,
            &LinearVelocity,
            &Slice,
            &Alliegance,
        ),
        Without<Destroyed>,
    >,
    time: Res<Time>,
    sq: SpatialQuery,
) {
    for (entity, transform, mut total_energy, equipped, linear_velocity, slice, alliegance) in
        parents.iter_mut()
    {
        // Get all entities that are weapons
        if let Some(entities) = equipped.equipped.get(&EquipmentTypeId::Weapon) {
            for weapon_entity in entities {
                // Get the weapon component attached
                if let Ok(mut weapon) = weapons.get_mut(*weapon_entity) {
                    // Do stuff depending on the weapon
                    match weapon.weapon_type {
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
                        } => {
                            if weapon.wants_to_fire {
                                // Check if weapon can fire
                                if weapon.last_fired + Duration::from_secs_f32(recoil)
                                    <= time.elapsed()
                                    && total_energy.consume(energy as f32).is_ok()
                                {
                                    for _ in 0..shots {
                                        let mut spread_angle = 0f32;
                                        if spread != 0f32 {
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

                                        // Spawn a projectile
                                        cmd.spawn((
                                            LockedAxes::new().lock_translation_z(),
                                            *transform,
                                            RigidBody::Dynamic,
                                            Mass(1f32),
                                            LinearVelocity(
                                                transform
                                                    .rotation
                                                    .mul_quat(Quat::from_axis_angle(
                                                        Vec3::Y,
                                                        (-added_angle.min(tracking).max(-tracking)
                                                            * (f32::from(direction)))
                                                            + spread_angle,
                                                    ))
                                                    .mul_vec3(-Vec3::Z * speed)
                                                    + linear_velocity.0,
                                            ),
                                            Lifetime {
                                                created: time.elapsed(),
                                                lifetime: Duration::from_secs_f32(lifetime),
                                            },
                                            Projectile { damage },
                                            *slice,
                                            alliegance.clone(),
                                            Sensor,
                                            Collider::sphere(radius),
                                            CollisionLayers {
                                                memberships: PhysicsCategory::Weapon.into(),
                                                filters: LayerMask::from([
                                                    PhysicsCategory::Craft,
                                                    PhysicsCategory::Structure,
                                                ]),
                                            },
                                        ));
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
                        } => {
                            if weapon.wants_to_fire
                                && total_energy
                                    .consume(energy_per_second * time.delta_seconds())
                                    .is_ok()
                            {
                                let mut rotation = *transform;
                                rotation.rotate_local_y(90f32.to_radians());

                                let rot = angle_with_tracking(&weapon, *transform, tracking, 0f32);

                                for hit in sq.shape_hits(
                                    &Collider::cuboid(width, width, range),
                                    transform.translation + (transform.down() * range * 0.5),
                                    default(),
                                    Dir3::new(rot.mul_vec3(transform.forward().into())).unwrap(),
                                    range,
                                    128,
                                    true,
                                    SpatialQueryFilter {
                                        mask: [PhysicsCategory::Craft, PhysicsCategory::Structure]
                                            .into(),
                                        excluded_entities: [entity].into(),
                                    },
                                ) {
                                    if let Ok(mut damage) = damage.get_mut(hit.entity) {
                                        **damage += damage_per_second * time.delta_seconds();
                                    }
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
