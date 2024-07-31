use bevy::prelude::*;
use big_brain::prelude::*;

use crate::prelude::*;

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

pub(crate) fn persue(
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
                                cmd.entity(actor.0).insert(Waypoint::Entity(*enemy));
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
                cmd.entity(actor.0).remove::<Waypoint>();
                None
            }
            _ => None,
        };
        if let Some(new_state) = new_state {
            *state = new_state;
        }
    }
}

pub(crate) fn retreat(
    mut cmd: Commands,
    mut actors: Query<(&Actor, &mut ActionState), With<actions::Retreat>>,
    actor_query: Query<&SpawnedFrom>,
) {
    for (actor, mut state) in actors.iter_mut() {
        let new_state = match state.as_ref() {
            ActionState::Requested => {
                // Set the waypoint back to the original spawnpoint
                Some(
                    actor_query
                        .get(actor.0)
                        .map(|spawned_from| {
                            cmd.entity(actor.0).insert(Waypoint::Entity(spawned_from.0));
                            ActionState::Executing
                        })
                        .ok()
                        .unwrap_or(ActionState::Failure),
                )
            }
            ActionState::Cancelled => Some(ActionState::Success),
            ActionState::Failure | ActionState::Success => {
                // remove the waypoint
                cmd.entity(actor.0).remove::<Waypoint>();
                None
            }
            _ => None,
        };
        if let Some(new_state) = new_state {
            *state = new_state;
        }
    }
}
