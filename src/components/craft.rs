//! Moveable thing

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::prelude::*;

/// Moveable thing
#[derive(Component)]
pub struct Craft {
    /// Top speed
    pub speed: f32,
    /// Top rotational speed
    pub rotation: f32,
    /// Braking force
    pub brake: f32,
    /// Acceleration
    pub acceleration: f32,
}

#[derive(Bundle)]
pub struct CraftBundle {
    pub controller: Controller,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub linear_damping: LinearDamping,
    pub mass: Mass,
    pub craft: Craft,
    pub locked_axes: LockedAxes,
    pub transform: Transform,
    pub alliegance: Alliegance,
    pub inventory: Inventory<'static>,
}

impl Default for CraftBundle {
    fn default() -> Self {
        Self {
            controller: Controller::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::sphere(0.5f32),
            mass: Mass(1f32),
            craft: Craft {
                speed: 20f32,
                rotation: 400f32,
                brake: 200f32,
                acceleration: 100f32,
            },
            locked_axes: LockedAxes::default()
                .lock_translation_z()
                .lock_rotation_y()
                .lock_rotation_x(),
            transform: Transform::default().looking_to(Dir3::X, Dir3::Z),
            alliegance: Alliegance {
                allies: Faction::empty(),
                enemies: Faction::empty(),
                faction: Faction::empty(),
            },
            inventory: Inventory::default(),
            linear_damping: LinearDamping::default(),
        }
    }
}
