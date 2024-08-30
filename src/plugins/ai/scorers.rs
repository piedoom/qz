use std::f32::consts::PI;

use bevy::prelude::*;
use big_brain::prelude::*;

use crate::prelude::*;

/// Scorer system for normalized facing value, where 0.0 is facing 180 degrees away, and 1.0 is facing exactly
///
/// # System overview
///
/// 1. Get all entities with the scorer
/// 2. Get the transform of the first enemy in range
/// 3. Find the turn angle to reach that enemy, and divide it by `PI`
pub(crate) fn facing_scorer(
    mut facings: Query<(&Actor, &mut Score), With<scorers::Facing>>,
    other: Query<(&InRange, &Transform)>,
    transforms: Query<&Transform>,
) {
    for (actor, mut score) in facings.iter_mut() {
        if let Ok((in_range, transform)) = other.get(actor.0) {
            score.set(match in_range.enemies.first() {
                Some(enemy) => {
                    let enemy_transform = transforms.get(*enemy).unwrap();
                    let (_, accuracy) =
                        transform.calculate_turn_angle(enemy_transform.translation.truncate());
                    // 1 is perfect accuracy, 0 is 180deg away
                    1f32 - f32::abs(accuracy / PI)
                }
                None => 0f32,
            });
        }
    }
}

/// 1.0 when on top of target, 0.0 when at or outside the range
pub(crate) fn target_in_range_scorer(
    mut actors: Query<(&Actor, &mut Score), With<scorers::TargetInRange>>,
    targets: Query<&Target>,
    equipped: Query<&Equipped>,
    weapons: Query<&Weapon>,
    transforms: Query<&Transform>,
) {
    // get the active weapon if it exists, otherwise the score is 0
    for (Actor(entity), mut score) in actors.iter_mut() {
        // reset score
        // score.set(0f32);

        // Check if a target is acquired
        if let Ok(Target(target_entity)) = targets.get(*entity) {
            let maybe_weapons = equipped.get(*entity).ok().map(|equipped| {
                equipped
                    .get_by_type(EquipmentTypeId::Weapon)
                    .flat_map(|weapon_entities| {
                        weapon_entities
                            .iter()
                            .filter_map(|weapon_entity| weapons.get(*weapon_entity).ok())
                    })
            });

            // get the weapon with the longest range by distance
            if let Some(weapons) = maybe_weapons {
                let longest_range = weapons.fold(0f32, |acc, b| {
                    let b_range = match b.weapon_type {
                        WeaponType::ProjectileWeapon { distance, .. }
                        | WeaponType::LaserWeapon {
                            range: distance, ..
                        } => distance,
                    };
                    if b_range > acc {
                        b_range
                    } else {
                        acc
                    }
                });

                // Get the range to the target as a score compared to the current weapon reach
                if let Ok([transform, target_transform]) =
                    transforms.get_many([*entity, *target_entity])
                {
                    let dist_sq = transform
                        .translation
                        .distance_squared(target_transform.translation);
                    let range_sq = longest_range.powi(2);

                    score.set((range_sq / dist_sq).clamp(0f32, 1f32));
                }
            }
        }
    }
}
