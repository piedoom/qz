use crate::prelude::*;
use bevy::prelude::*;
use bevy_htnp::{prelude::HtnAgentState, tasks::TaskRegistry};
use bevy_turborand::prelude::*;

pub(super) fn generate_task_registry() -> TaskRegistry {
    use super::*;
    let mut tasks = TaskRegistry::new();
    tasks.task::<task::Search, _>(
        task::Search::NAME,
        Requirements::new()
            .req_equals(Requirement::EnemiesInView.name(), false)
            .req_equals(Requirement::TargetInWeaponsRange.name(), false)
            .req_equals(Requirement::TargetDestroyed.name(), false)
            .build(),
        WorldState::new()
            .add(Requirement::EnemiesInView.name(), true)
            .add(Requirement::TargetInWeaponsRange.name(), false)
            .build(),
        1.0,
    );
    tasks.task::<task::Persue, _>(
        task::Persue::NAME,
        Requirements::new()
            .req_equals(Requirement::EnemiesInView.name(), true)
            .req_equals(Requirement::TargetInWeaponsRange.name(), false)
            .build(),
        WorldState::new()
            .add(Requirement::EnemiesInView.name(), true)
            .add(Requirement::TargetInWeaponsRange.name(), true)
            .build(),
        1.0,
    );
    tasks.task::<task::Attack, _>(
        task::Attack::NAME,
        Requirements::new()
            .req_equals(Requirement::EnemiesInView.name(), true)
            .req_equals(Requirement::TargetInWeaponsRange.name(), true)
            .build(),
        WorldState::new()
            .add(Requirement::TargetDestroyed.name(), true)
            .build(),
        1.0,
    );
    tasks
}

pub(crate) fn search(
    mut actors: Query<(Entity, &HtnAgentState, &mut Waypoint), With<task::Search>>,
    mut rng: ResMut<GlobalRng>,
    global_transforms: Query<&GlobalTransform>,
) {
    for (entity, state, mut waypoint) in actors.iter_mut() {
        if *state != HtnAgentState::Running {
            continue;
        }
        dbg!("search");
        // Get desired position, or none if none is set
        let maybe_waypoint_pos = match *waypoint {
            Waypoint::Entity(tar) => Some(
                global_transforms
                    .get(tar)
                    .unwrap()
                    .compute_transform()
                    .translation
                    .truncate(),
            ),
            Waypoint::Position(pos) => Some(pos),
            Waypoint::None => None,
        };

        // Current Vec2 of the actor
        let current_pos = global_transforms
            .get(entity)
            .unwrap()
            .compute_transform()
            .translation
            .truncate();

        match maybe_waypoint_pos {
            None => {
                // Generate a new random position to go to
                let add_x = (rng.f32_normalized() - 0.5) * 16f32;
                let add_y = (rng.f32_normalized() - 0.5) * 16f32;
                *waypoint = Waypoint::Position(current_pos + Vec2::new(add_x, add_y))
            }
            Some(waypoint_pos) => {
                // If sufficiently close to the waypoint, clear
                if current_pos.distance_squared(waypoint_pos) < 2f32.powi(2) {
                    *waypoint = Waypoint::None
                }
            }
        }
    }
}

pub(crate) fn persue(
    mut actors: Query<(&mut Waypoint, &HtnAgentState, &InRange), With<task::Persue>>,
) {
    for (mut waypoint, state, in_range) in actors.iter_mut() {
        if *state != HtnAgentState::Running {
            continue;
        }
        dbg!("persue");
        match in_range.enemies.first() {
            Some(first) => {
                *waypoint = Waypoint::Entity(*first);
            }
            None => {
                *waypoint = Waypoint::None;
            }
        }
    }
}

pub(crate) fn attack(
    mut actors: Query<(&mut Waypoint, &HtnAgentState, &InRange, &Equipped), With<task::Attack>>,
    mut weapons: Query<&mut Weapon>,
) {
    for (mut waypoint, state, in_range, equipped) in actors.iter_mut() {
        if *state != HtnAgentState::Running {
            continue;
        }
        dbg!("attack");
        if let Some(first) = in_range.enemies.first() {
            *waypoint = Waypoint::Entity(*first);
            // set weapons to fire
            if let Some(weapon_entities) = equipped.equipped.get(&EquipmentTypeId::Weapon) {
                for weapon_entity in weapon_entities.iter() {
                    if let Ok(mut weapon) = weapons.get_mut(*weapon_entity) {
                        weapon.wants_to_fire = true;
                    }
                }
            }
        }
    }
}
