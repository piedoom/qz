use bevy::prelude::*;
use big_brain::prelude::*;

use crate::prelude::*;

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
