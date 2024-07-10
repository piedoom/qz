use std::{f32::consts::PI, ops::RangeInclusive};

use avian3d::prelude::PhysicsLayer;
use bevy::prelude::*;

/// Additional methods for the [`RangeInclusive`] type
pub trait RangeInclusiveExt<T> {
    /// Perform a linear interpolation
    ///
    /// # Arguments
    ///
    /// * `at` - a normalized value describing the interpolation factor
    fn lerp(&self, at: f32) -> T;
}

impl RangeInclusiveExt<f32> for RangeInclusive<f32> {
    fn lerp(&self, at: f32) -> f32 {
        let delta = self.end() - self.start();
        self.start() + (at * delta)
    }
}

pub trait TransformExt {
    /// Default with Z-up
    fn default_z() -> Transform {
        Transform::default().looking_to(Dir3::X, Dir3::Z)
    }

    /// Calculate a direction needed to turn to face another transform along with facing accuracy
    fn calculate_turn_angle(&self, other: impl Into<Transform>) -> (RotationDirection, f32);
}

impl TransformExt for Transform {
    fn calculate_turn_angle(&self, other: impl Into<Transform>) -> (RotationDirection, f32) {
        let other = other.into();
        // get the forward vector in 2D
        let forward = (self.rotation * Vec3::Z).xy();

        // get the vector from the ship to the enemy ship in 2D and normalize it.
        let to_other = (self.translation.xy() - other.translation.xy()).normalize();

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

pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
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

#[derive(PhysicsLayer)]
pub enum PhysicsCategory {
    Structure,
    Craft,
    Weapon,
    Item,
}

#[derive(Component, Deref, DerefMut)]
pub struct WorldLayer(usize);
