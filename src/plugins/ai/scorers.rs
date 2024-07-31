use std::f32::consts::PI;

use bevy::prelude::*;
use big_brain::prelude::*;

use crate::prelude::*;

pub(crate) fn danger_scorer(
    mut dangers: Query<(&Actor, &scorers::Danger, &mut Score)>,
    other: Query<(&InRange, &Transform)>,
    transforms: Query<&Transform>,
) {
    for (actor, danger, mut score) in dangers.iter_mut() {
        if let Ok((in_range, transform)) = other.get(actor.0) {
            // Find closest enemy
            score.set(match in_range.enemies.first() {
                Some(enemy) => match transforms.get(*enemy) {
                    Ok(enemy_transform) => {
                        let distance_to_enemy_squared = transform
                            .translation
                            .distance_squared(enemy_transform.translation);
                        danger.score(distance_to_enemy_squared)
                    }
                    Err(_) => 0f32,
                },
                None => 0f32,
            });
        }
    }
}

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
                    1f32 - (accuracy / PI)
                }
                None => 0f32,
            });
        }
    }
}
