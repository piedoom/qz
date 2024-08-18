use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A weapon that can fire
#[derive(Debug, Clone, Reflect, Component, Serialize, Deserialize, PartialEq)]
pub struct Weapon {
    /// If `true`, this weapon will attempt to fire. This is not reset to false implicitly
    #[serde(skip)]
    pub wants_to_fire: bool,
    /// If `Some`, this weapon is already firing. The inner data is the timestamp that the weapon began firing
    #[serde(skip)]
    pub firing: Option<Duration>,
    #[serde(skip)]
    /// The optional target coordinate of this weapon, used for mouse aiming and tracking
    pub target: Option<Vec3>,
    /// The timestamp of when this weapon was last fired, used for calculating recoil
    #[serde(skip)]
    pub last_fired: Duration,
    /// The specific weapon type will further influence how this weapon behaves
    pub weapon_type: WeaponType,
}
/// Specific weapon type
#[derive(Debug, Clone, Reflect, Serialize, Deserialize, PartialEq)]
pub enum WeaponType {
    /// Kinetic weapons that shoot projectiles at things
    ProjectileWeapon {
        /// Allows the projectile to be fired at a direction other than straight ahead,
        /// where PI is full sweeping coverage
        #[serde(default)]
        tracking: f32,
        /// Speed of projectile
        speed: f32,
        /// Duration between new projectile shots in seconds
        recoil: f32,
        /// Cone in radians of potential spread
        #[serde(default)]
        spread: f32,
        /// Shots to fire at once
        shots: usize,
        /// Damage to inflict per shot
        damage: usize,
        /// Radius of the projectile to be fired
        radius: f32,
        /// Lifetime in seconds
        lifetime: f32,
        /// Energy consumed by each shot
        energy: usize,
        /// Model for projectile
        projectile_model: String,
    },
    /// A laser
    LaserWeapon {
        /// Maximum (absolute) tracking angle from 0-PI
        #[serde(default)]
        tracking: f32,
        /// Damage to inflict per second of contact
        damage_per_second: f32,
        /// Energy to consume per second of firing
        energy_per_second: f32,
        /// Length of the laser
        range: f32,
        /// Width of the laser
        width: f32,
        /// Energy for this laser to be activated
        activation_energy: f32,
        /// Time to overheat
        heat_per_second: f32,
        /// Recovery per second
        cooling_per_second: f32,
        /// Beam color
        color: (f32, f32, f32),
    },
}

/// Shot from a [`ProjectileWeapon`]
#[derive(Clone, Reflect, Component)]
pub struct Projectile {
    /// Damage to inflict on hit
    pub damage: usize,
}

/// Fired from a [`LaserWeapon`]
#[derive(Clone, Reflect, Component)]
pub struct Laser {
    /// Damage to inflict per second on hit
    pub damage_per_second: f32,
    /// Length of the laser
    pub range: f32,
    /// Width of the laser
    pub width: f32,
}
