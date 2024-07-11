use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Component, Reflect, Serialize, Deserialize)]
pub struct Weapon {
    #[serde(skip)]
    pub wants_to_fire: bool,
    #[serde(skip)]
    pub target: Option<Vec3>,
    #[serde(skip)]
    pub last_fired: Duration,
    pub weapon_type: WeaponType,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub enum WeaponType {
    ProjectileWeapon {
        /// Allows the projectile to be fired at a direction other than straight ahead,
        /// where PI is full sweeping coverage
        #[serde(default)]
        tracking: f32,
        /// Speed of projectile
        speed: f32,
        /// Duration between new projectile shots in seconds
        recoil: f32,
        // Cone in radians of potential spread
        #[serde(default)]
        spread: f32,
        // Shots to fire at once
        shots: usize,
        damage: usize,
        radius: f32,
        /// Lifetime in seconds
        lifetime: f32,
        /// Energy consumed by each shot
        energy: usize,
    },
}

#[derive(Clone, Reflect, Component)]
pub struct Projectile {
    pub damage: usize,
}
