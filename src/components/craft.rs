//! Moveable thing

use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Component, Reflect)]
pub struct Destroyed;

#[derive(Component, Reflect, Deref, DerefMut)]
pub struct Health(pub usize);

/// Damage inflicted. Used in tandem with [`Health`]. Damage is a float instead of an integer, as
/// repairs may repair fractional amounts
#[derive(Component, Reflect, Deref, DerefMut, Default)]
pub struct Damage(f32);

// #[derive(Component, Reflect, Deref, DerefMut, Default)]
// pub struct ShieldStore(f32);

// #[derive(Component, Reflect, Deref, DerefMut, Default)]
// pub struct EnergyStore(f32);

impl From<usize> for Health {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<f32> for Damage {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

/// Moveable thing
#[derive(Debug, Clone, Component, Reflect, Asset, Serialize, Deserialize)]
pub struct Craft {
    /// Name
    pub name: String,
    /// Top speed
    pub speed: f32,
    /// Top rotational speed
    pub rotation: f32,
    /// Braking force
    pub brake: f32,
    /// Acceleration
    pub acceleration: f32,
    /// Base health
    pub health: usize,
    /// Hitbox size
    pub size: f32,
    /// Base mass
    pub mass: f32,
    /// Inventory capacity
    pub capacity: usize,
    pub value: usize,
}

#[derive(Bundle)]
pub struct CraftBundle {
    pub energy: Energy,
    pub controller: Controller,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub linear_damping: LinearDamping,
    pub mass: Mass,
    pub craft: Craft,
    pub locked_axes: LockedAxes,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub alliegance: Alliegance,
    pub inventory: Inventory,
    pub equipped: EquippedBuilder,
    pub collision_layers: CollisionLayers,
    pub credits: Credits,
    pub rotation: Rotation,
}

impl Default for CraftBundle {
    fn default() -> Self {
        Self {
            energy: Energy::default(),
            controller: Controller::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::sphere(0.5f32),
            mass: Mass(1f32),
            craft: Craft {
                speed: 10f32,
                rotation: 400f32,
                brake: 200f32,
                acceleration: 40f32,
                health: 64,
                size: 1f32,
                mass: 100f32,
                capacity: 100,
                name: "craft".to_string(),
                value: 1000,
            },
            locked_axes: LockedAxes::default().lock_translation_z(),
            // .lock_rotation_y() // TODO: Avian bug?
            // .lock_rotation_x(),
            alliegance: Alliegance::default(),
            inventory: Inventory::default(),
            linear_damping: LinearDamping::default(),
            equipped: EquippedBuilder::default(),
            collision_layers: CollisionLayers {
                memberships: LayerMask::from([PhysicsCategory::Craft]),
                filters: LayerMask::from([PhysicsCategory::Craft, PhysicsCategory::Weapon]),
            },
            credits: default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::IDENTITY,
            rotation: Rotation(Transform::default_z().rotation),
        }
    }
}
