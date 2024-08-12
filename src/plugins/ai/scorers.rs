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

// /// 1.0 when on top of target, 0.0 when at or outside the range
// pub(crate) fn target_in_range_scorer(
//     mut actors: Query<(&Actor, &mut Score), With<scorers::TargetInRange>>,
//     targeting:
//     transforms: Query<&Transform>,
// ) {
// }
