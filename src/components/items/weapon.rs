use std::time::Duration;

use bevy::prelude::*;

#[derive(Clone, Component)]
pub struct Weapon {
    pub wants_to_fire: bool,
    pub target: Option<Vec3>,
    pub last_fired: Duration,
    pub weapon_type: WeaponType,
}

#[derive(Clone)]
pub enum WeaponType {
    Projectile {
        /// Allows the projectile to be fired at a direction other than straight ahead,
        /// where PI is full sweeping coverage
        tracking: f32,
        /// Speed of projectile
        speed: f32,
        /// Duration between new projectile shots
        recoil: Duration,
        // Cone in radians of potential spread
        spread: f32,
        // Shots to fire at once
        shots: usize,
        damage: usize,
        radius: f32,
        lifetime: Duration,
    },
}

#[derive(Clone, Component)]
pub struct Projectile {
    pub damage: usize,
}
