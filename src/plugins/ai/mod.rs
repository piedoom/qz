use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use big_brain::BigBrainSet;

/// Utility AI actions
mod actions;

/// Utility AI scorers
mod scorers;

/// Plugin for Utility AI logic
pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_in_range, move_towards_waypoint))
            // Scorers
            .add_systems(
                PreUpdate,
                (scorers::facing_scorer).in_set(BigBrainSet::Scorers),
            )
            // Actions
            .add_systems(
                PreUpdate,
                (actions::attack, actions::persue).in_set(BigBrainSet::Actions),
            );
    }
}

/// Given a range, update all entities marked as enemies or allies
fn update_in_range(
    mut in_range: Query<(Entity, &mut InRange, &Alliegance, &Transform), Without<Destroyed>>,
    query: SpatialQuery,
    other: Query<(Entity, &Transform, &Alliegance), Without<Destroyed>>,
) {
    for (entity, mut in_range, alliegance, transform) in in_range.iter_mut() {
        // Reset all
        in_range.clear();

        // Cast a shape in our distance of reach
        let mut collisions = query
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
            .collect::<Vec<_>>();

        // Sort by distance. This will actually get reversed when we push it into two other vecs, so its reversed direction here
        collisions.sort_by(|a, b| {
            transform
                .translation
                .distance_squared(b.1.translation)
                .partial_cmp(&transform.translation.distance_squared(a.1.translation))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (obj_entity, _, obj_alliegance) in collisions.iter() {
            if alliegance.allies.contains(&obj_alliegance.faction) {
                in_range.allies.push(*obj_entity);
            } else if alliegance.enemies.contains(&obj_alliegance.faction) {
                in_range.enemies.push(*obj_entity);
            }
        }
    }
}

/// Move controllers towards the given waypoint
///
/// # System overview
///
/// 1. Get entities with a controller and a waypoint
/// 2. If the waypoint is set, find the turn angle needed to face the target
/// 3. Once the angle is within a certain tolerance, thrust forwards
pub(crate) fn move_towards_waypoint(
    mut query: Query<(Entity, &mut Controller, &Waypoint)>,
    transforms: Query<&Transform>,
) {
    for (entity, mut controller, waypoint) in query.iter_mut() {
        if let Some(target_transform) = match waypoint {
            Waypoint::Entity(e) => transforms.get(*e).ok().cloned(),
            Waypoint::Position(p) => Some(Transform::from_translation(p.extend(0f32))),
        } {
            if let Ok(transform) = transforms.get(entity) {
                let (turn, angle) =
                    transform.calculate_turn_angle(target_transform.translation.truncate());

                controller.angular_thrust = turn.into();
                if angle < 20f32.to_radians() {
                    controller.thrust = 1f32;
                    controller.angular_thrust = 0f32;
                } else {
                    controller.thrust = 0f32
                }
            }
        }
    }
}
