use std::ops::RangeInclusive;

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
}

impl TransformExt for Transform {}
