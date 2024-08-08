use std::ops::RangeInclusive;

use avian3d::prelude::PhysicsLayer;
use bevy::prelude::*;

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

/// Extention methods for asset ergonomics
pub trait LibraryExt {
    /// Get a creature (`*.creature.ron`) by name string
    fn creature(&self, name: impl AsRef<str>) -> Option<Handle<Creature>>;
    /// Get a craft (`*.craft.ron`) by name string
    fn craft(&self, name: impl AsRef<str>) -> Option<Handle<Craft>>;
    /// Get a building (`*.building.ron`) by name string
    fn building(&self, name: impl AsRef<str>) -> Option<Handle<Building>>;
    /// Get an item (`*.item.ron`) by name string
    fn item(&self, name: impl AsRef<str>) -> Option<Handle<Item>>;
}

impl<'a> LibraryExt for Res<'a, Library> {
    fn creature(&self, name: impl AsRef<str>) -> Option<Handle<Creature>> {
        self.creatures
            .get(&format!("creatures/{}.creature.ron", name.as_ref()))
            .cloned()
    }

    fn craft(&self, name: impl AsRef<str>) -> Option<Handle<Craft>> {
        self.crafts
            .get(&format!("crafts/{}.craft.ron", name.as_ref()))
            .cloned()
    }

    fn building(&self, name: impl AsRef<str>) -> Option<Handle<Building>> {
        self.buildings
            .get(&format!("buildings/{}.building.ron", name.as_ref()))
            .cloned()
    }

    fn item(&self, name: impl AsRef<str>) -> Option<Handle<Item>> {
        self.items
            .get(&format!("items/{}.ron", name.as_ref()))
            .cloned()
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
