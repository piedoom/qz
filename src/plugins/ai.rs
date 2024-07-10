use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (fly_towards_enemy, update_in_range));
    }
}

fn fly_towards_enemy(
    mut npcs: Query<
        (&mut Controller, &Transform, &InRange, Option<&Children>),
        (With<Npc>, Without<Destroyed>),
    >,
    transforms: Query<&Transform>,
    mut weapons: Query<&mut Weapon>,
) {
    for (mut controller, transform, in_range, maybe_children) in npcs.iter_mut() {
        // TODO: Don't retarget every frame
        // Get closest enemy
        let (closest_enemy, _) = in_range.enemies.iter().fold(
            (Entity::PLACEHOLDER, f32::MAX),
            |(acc_entity, acc_distance), e| {
                if let Ok(obj_transform) = transforms.get(*e) {
                    let distance_squared = obj_transform
                        .translation
                        .distance_squared(transform.translation);
                    if distance_squared < acc_distance {
                        (*e, distance_squared)
                    } else {
                        (acc_entity, acc_distance)
                    }
                } else {
                    (acc_entity, acc_distance)
                }
            },
        );

        if let Ok(enemy_transform) = transforms.get(closest_enemy) {
            let (turn, accuracy) = transform.calculate_turn_angle(*enemy_transform);
            if accuracy < 0.05 && maybe_children.is_some() {
                for child in maybe_children.unwrap() {
                    if let Ok(mut weapon) = weapons.get_mut(*child) {
                        weapon.wants_to_fire = true;
                    }
                }
            }
            // if the dot product is approximately 1.0 then already facing and
            // we can early out.
            if accuracy < f32::EPSILON {
                continue;
            }

            controller.angular_thrust = turn.into();
            if accuracy < 0.5 {
                controller.thrust = 1f32
            } else {
                controller.thrust = 0f32
            }
        }
    }
}

/// Given a range, update all entities marked as enemies or allies
fn update_in_range(
    mut in_range: Query<(Entity, &mut InRange, &Alliegance, &Transform), Without<Destroyed>>,
    query: SpatialQuery,
    other: Query<(Entity, &Alliegance), Without<Destroyed>>,
) {
    for (entity, mut in_range, alliegance, transform) in in_range.iter_mut() {
        // Reset all
        in_range.clear();

        // Cast a shape in our distance of reach
        for (obj_entity, obj_alliegance) in query
            .shape_intersections(
                &Collider::cylinder(in_range.range, 1f32),
                transform.translation,
                Transform::default_z().rotation,
                SpatialQueryFilter {
                    mask: LayerMask::from([PhysicsCategory::Craft, PhysicsCategory::Structure]),
                    excluded_entities: [entity].into(),
                },
            )
            .iter()
            .filter_map(|r| other.get(*r).ok())
        {
            if alliegance.allies.contains(obj_alliegance.faction) {
                in_range.allies.push(obj_entity);
            } else if alliegance.enemies.contains(obj_alliegance.faction) {
                in_range.enemies.push(obj_entity);
            }
        }
    }
}
