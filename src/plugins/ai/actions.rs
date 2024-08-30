use std::f32::consts::TAU;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_turborand::*;
use big_brain::prelude::*;

/// System to controll attacking entities
///
/// # System overview
///
/// 1. Get all entities that want to attack
/// 2. Loop through those entities' children
/// 3. Check if child is a weapon. If so, set `wants_to_fire` to `true`
/// 4. When finished attacking, set `wants_to_fire` to `false`
pub(crate) fn attack(
    mut actors: Query<(&Actor, &mut ActionState), With<actions::Attack>>,
    mut weapons: Query<&mut Weapon>,
    children: Query<&Children>,
) {
    let mut set_wants_to_fire = |actor: Actor, wants_to_fire: bool| {
        if let Ok(children) = children.get(actor.0) {
            for child in children.iter() {
                // Attempt to get a weapon from the child entity
                if let Ok(mut weapon) = weapons.get_mut(*child) {
                    weapon.wants_to_fire = wants_to_fire;
                }
            }
        }
    };

    for (actor, mut state) in actors.iter_mut() {
        let new_state = match state.as_ref() {
            ActionState::Requested => {
                set_wants_to_fire(*actor, true);
                Some(ActionState::Executing)
            }
            ActionState::Executing => {
                set_wants_to_fire(*actor, true);
                None
            }
            ActionState::Cancelled => Some(ActionState::Success),
            ActionState::Failure | ActionState::Success => {
                set_wants_to_fire(*actor, false);
                None
            }
            _ => None,
        };
        if let Some(new_state) = new_state {
            *state = new_state;
        }
    }
}

/// System that will fly a [`Controller`] to a given entity [`Waypoint`], if it exists
///
/// # System overview
///
/// 1. Get all entities that want to persue
/// 2. If state is requested, set the waypoint to the nearest enemy in range
/// 3. When cancelled or otherwise ended, the waypoint is removed
pub(crate) fn persue_enemies(
    mut cmd: Commands,
    mut actors: Query<(&Actor, &mut ActionState), With<actions::Persue>>,
    actor_query: Query<&InRange>,
) {
    // Loop through all actors that want to persue
    for (actor, mut state) in actors.iter_mut() {
        let new_state = match state.as_ref() {
            ActionState::Requested => {
                // Set waypoint to nearest enemy
                Some(
                    actor_query
                        .get(actor.0)
                        .map(|in_range| {
                            in_range.enemies.first().map(|enemy| {
                                cmd.entity(actor.0)
                                    .insert(Waypoint::Entity(*enemy))
                                    .insert(Target(*enemy));

                                ActionState::Executing
                            })
                        })
                        .ok()
                        .flatten()
                        .unwrap_or(ActionState::Failure),
                )
            }
            ActionState::Cancelled => Some(ActionState::Success),
            ActionState::Failure | ActionState::Success => {
                // remove the waypoint
                cmd.entity(actor.0).remove::<Waypoint>().remove::<Target>();
                None
            }
            _ => None,
        };
        if let Some(new_state) = new_state {
            *state = new_state;
        }
    }
}

pub(crate) fn idle(
    mut cmd: Commands,
    mut actors: Query<(&Actor, &mut ActionState), With<actions::Idle>>,
    mut rng: ResMut<GlobalRng>,
    global_transforms: Query<&GlobalTransform>,
    waypoints: Query<&Waypoint>,
    spawned_from: Query<&SpawnedFrom>,
    // transforms: Query<&Transform>,
) {
    const RADIUS_FROM: f32 = 12f32;
    const TOLERANCE_RADIUS: f32 = 1f32;
    for (Actor(entity), mut state) in actors.iter_mut() {
        let new_state = match state.as_ref() {
            ActionState::Requested => {
                cmd.entity(*entity).insert(Waypoint::Position(
                    global_transforms
                        .get(*entity)
                        .unwrap()
                        .compute_transform()
                        .translation
                        .truncate(),
                ));
                Some(ActionState::Executing)
            }
            ActionState::Executing => {
                let entity_transform = global_transforms.get(*entity).unwrap().compute_transform();
                let maybe_waypoint_transform = waypoints
                    .get(*entity)
                    .and_then(|w| match w {
                        Waypoint::Entity(e) => global_transforms
                            .get(*e)
                            .copied()
                            .map(|x| x.compute_transform()),
                        Waypoint::Position(p) => Ok(Transform::from_translation(p.extend(0f32))),
                    })
                    .ok();

                if let Some(waypoint_transform) = maybe_waypoint_transform {
                    if entity_transform
                        .translation
                        .distance_squared(waypoint_transform.translation)
                        <= TOLERANCE_RADIUS
                    {
                        let mut set_random_from_transform =
                            |transform: &Transform, rng: &mut GlobalRng| {
                                let mut pos =
                                    Transform::default_z().with_translation(transform.translation);
                                pos.rotate_z(rng.f32_normalized() * TAU);
                                let point =
                                    pos.forward().as_vec3() * rng.f32_normalized() * RADIUS_FROM;
                                cmd.entity(*entity)
                                    .insert(Waypoint::Position(point.truncate()));
                            };
                        // Find a random waypoint to move towards
                        match spawned_from.get(*entity) {
                            // Find a random point to set as a waypoint. If this entity was spawned
                            // from another entity, we'll attempt to find a point close to the spawner
                            Ok(SpawnedFrom(spawned_from)) => {
                                let spawned_transform = global_transforms
                                    .get(*spawned_from)
                                    .unwrap()
                                    .compute_transform();
                                set_random_from_transform(&spawned_transform, &mut rng);
                            }
                            Err(_) => {
                                // Go to a random relative position
                                let actor_transform =
                                    global_transforms.get(*entity).unwrap().compute_transform();
                                set_random_from_transform(&actor_transform, &mut rng);
                            }
                        }
                    }
                }
                None
            }
            ActionState::Cancelled => Some(ActionState::Success),
            ActionState::Failure | ActionState::Success => {
                // remove the waypoint
                cmd.entity(*entity).remove::<Waypoint>();
                None
            }
            _ => None,
        };
        if let Some(new_state) = new_state {
            *state = new_state;
        }
    }
}
