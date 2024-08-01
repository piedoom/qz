use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Reflect, Component, Serialize, Deserialize, PartialEq)]
pub struct Weapon {
    #[serde(skip)]
    pub wants_to_fire: bool,
    #[serde(skip)]
    pub target: Option<Vec3>,
    #[serde(skip)]
    pub last_fired: Duration,
    pub weapon_type: WeaponType,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize, PartialEq)]
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
    LaserWeapon {
        #[serde(default)]
        tracking: f32,
        damage_per_second: f32,
        energy_per_second: f32,
        range: f32,
        width: f32,
    },
}

#[derive(Clone, Reflect, Component)]
pub struct Projectile {
    pub damage: usize,
}

#[derive(Clone, Reflect, Component)]
pub struct Laser {
    pub damage_per_second: f32,
    pub range: f32,
    pub width: f32,
}
