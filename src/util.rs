use std::{
    ops::{RangeBounds, RangeInclusive},
    path::PathBuf,
};

use avian3d::prelude::PhysicsLayer;
use bevy::{
    asset::AssetPath,
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use bevy_etcetera::Directories;

use crate::{prelude::*, resources::Library};

/// Additional methods for the [`RangeInclusive`] type
pub trait RangeInclusiveExt<T> {
    /// Perform a linear interpolation
    ///
    /// # Arguments
    ///
    /// * `at` - a normalized value describing the interpolation factor
    fn lerp(&self, at: f32) -> f32;
}

impl RangeInclusiveExt<f32> for RangeInclusive<f32> {
    fn lerp(&self, at: f32) -> f32 {
        let delta = *self.end() - *self.start();
        (*self.start() + (at * delta)).clamp(0.0, 1.0)
    }
}

/// Extension methods for [`TransformBundle`]
pub trait TransformBundleExt {
    /// [`TransformBundle::default`], except with Z-up instead of Y-up, as god intended
    fn default_z() -> TransformBundle {
        TransformBundle::from_transform(Transform::default().looking_to(Dir3::X, Dir3::Z))
    }
}
impl TransformBundleExt for TransformBundle {}

/// Extension methods for [`Transform`]
pub trait TransformExt {
    /// [`Transform::default`], except with Z-up instead of Y-up
    fn default_z() -> Transform {
        Transform::default().looking_to(Dir3::X, Dir3::Z)
    }

    /// Creates a `Transform` with a given `translantion` point and `rotation` in radians
    fn z_from_parts(translation: &Vec2, rotation: &f32) -> Transform {
        let mut t = Transform::default_z().with_translation(translation.extend(0f32));
        t.rotate_z(*rotation);
        t
    }

    /// Calculate a direction needed to turn to face another transform along with facing accuracy
    fn calculate_turn_angle(&self, other: impl Into<Vec2>) -> (RotationDirection, f32);
}

impl TransformExt for Transform {
    fn calculate_turn_angle(&self, other: impl Into<Vec2>) -> (RotationDirection, f32) {
        let other = other.into();
        // get the forward vector in 2D
        let forward = (self.rotation * Vec3::Z).xy();

        // get the vector from the ship to the enemy ship in 2D and normalize it.
        let to_other = (self.translation.xy() - other).normalize();

        // get the dot product between the enemy forward vector and the direction to the player.
        let forward_dot_other = forward.dot(to_other);
        let accuracy = (forward_dot_other - 1.0).abs();

        // if the dot product is approximately 1.0 then already facing and
        // we can early out.
        if accuracy < f32::EPSILON {
            return (RotationDirection::None, 0.0);
        }

        // get the right vector of the ship in 2D (already unit length)
        let right = (self.rotation * Vec3::X).xy();
        let right_dot_other = right.dot(to_other);
        let rotation_sign = -f32::copysign(1.0, right_dot_other);
        let angle = forward.angle_between(to_other) * -rotation_sign;
        match rotation_sign > 0f32 {
            true => (RotationDirection::Clockwise, angle),
            false => (RotationDirection::CounterClockwise, angle),
        }
    }
}

/// Float extensions
pub trait F32Ext<R>
where
    R: RangeBounds<f32>,
{
    /// Get this value as a normalized value between the given range
    fn normalize(&self, in_range: R) -> f32;
}

impl<R> F32Ext<R> for f32
where
    R: RangeBounds<f32>,
{
    fn normalize(&self, in_range: R) -> f32 {
        let end = match in_range.end_bound() {
            std::ops::Bound::Included(x) | std::ops::Bound::Excluded(x) => x,
            std::ops::Bound::Unbounded => &f32::INFINITY,
        };
        let start = match in_range.start_bound() {
            std::ops::Bound::Included(x) | std::ops::Bound::Excluded(x) => x,
            std::ops::Bound::Unbounded => &0f32,
        };
        let total_range = end - start;
        let unclamped = (self - start) / total_range;
        unclamped.clamp(0f32, 1f32)
    }
}

/// Direction of a rotation, typically a rotation that is made to face another position
pub enum RotationDirection {
    /// Clockwise rotation
    Clockwise,
    /// Counter clockwise rotation
    CounterClockwise,
    /// No rotation (e.g., already facing)
    None,
}

impl From<RotationDirection> for f32 {
    fn from(val: RotationDirection) -> Self {
        match val {
            RotationDirection::Clockwise => 1f32,
            RotationDirection::CounterClockwise => -1f32,
            RotationDirection::None => 0f32,
        }
    }
}

/// Broad physics lasers so that we can more easily sense collisions
#[derive(PhysicsLayer)]
pub enum PhysicsCategory {
    /// Buildings like a station or spawner
    Structure,
    /// Vehicles like the player or enemy creatures
    Craft,
    /// Weapon effects, like projectiles and lasers
    Weapon,
    /// Items, like chests and floating credits
    Item,
}

/// Piped system to handle errors and show them as in-game toasts
pub fn handle_errors<E>(In(result): In<Result<(), E>>, mut errors: EventWriter<GameError>)
where
    E: std::fmt::Display + Into<GameError>,
{
    if let Err(e) = result {
        eprintln!("{e}");
        errors.send(e.into());
    }
}

/// Numerator over denominator
pub type Chance = (usize, usize);

pub trait Builder: Clone + Component {
    type Output: Bundle;
    fn on_add(mut world: DeferredWorld, entity: Entity, _component_id: ComponentId) {
        // Replace this builder with the actual component, then remove
        let builder = world.get::<Self>(entity).cloned().unwrap();
        let drops = Self::into_output(builder, world.resource::<Library>());
        let mut cmd = world.commands();
        let mut entity_builder = cmd.entity(entity);
        entity_builder.insert(drops);
        entity_builder.remove::<Self>();
    }

    fn from_output(output: Self::Output) -> Self;
    fn into_output(builder: Self, library: &Library) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_f32() {
        assert_eq!(0.5f32, 0.5f32.normalize(0f32..=1f32));
        assert_eq!(0.7f32, 0.7f32.normalize(0f32..=1f32));
        assert_eq!(0.7f32, 0.3f32.normalize(1f32..=0f32));
        assert_eq!(0.7f32, (-0.3f32).normalize(-1f32..=0f32));
    }
}
