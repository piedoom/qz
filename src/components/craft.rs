use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
/// Marker component for a destroyed entity
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Copy)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Destroyed;

/// Total alotted hitpoints
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Copy)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Health {
    base: usize,
    #[serde(skip)]
    bonus: usize,
}

impl Health {
    pub fn get(&self) -> usize {
        self.base + self.bonus
    }

    pub fn new(base: usize) -> Self {
        Self { base, bonus: 0 }
    }

    pub fn add_bonus(&mut self, bonus: usize) {
        self.bonus += bonus;
    }

    pub fn remove_bonus(&mut self, bonus_to_remove: usize) {
        self.bonus = self.bonus.checked_sub(bonus_to_remove).unwrap_or_default();
    }
}

/// Damage inflicted. Used in tandem with [`Health`]. Damage is a float instead of an integer, as
/// repairs may repair fractional amounts
#[derive(Component, Reflect, Clone, Copy, Deref, DerefMut, Default, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Damage(f32);

// #[derive(Component, Reflect, Deref, DerefMut, Default)]
// pub struct ShieldStore(f32);

// #[derive(Component, Reflect, Deref, DerefMut, Default)]
// pub struct EnergyStore(f32);

impl From<f32> for Damage {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

/// Moveable thing
#[derive(Debug, Reflect, Clone, Component, Asset, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
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
    /// Craft value
    pub value: usize,
}

#[derive(Debug, Reflect, Clone, Component, Asset, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct CraftName(pub String);

/// Common shape for a moving craft, such as for AI or the player
#[derive(Bundle)]
pub struct CraftBundle {
    /// Starting [`Energy`]
    pub energy: Energy,
    /// A [`Controller`] allows this craft to move
    pub controller: Controller,
    /// Always will be set to `Dynamic`
    pub rigid_body: RigidBody,
    /// As a convention, use a ball
    pub collider: Collider,
    /// Added so that we can control it later, as it isn't auto-added with the collider
    pub linear_damping: LinearDamping,
    /// Mass will be calculated based on craft and inventory weight
    pub mass: Mass,
    /// The actual [`Craft`] specification from our assets
    pub craft: Craft,
    /// Axes locked to only move in the Z plane with rotation only on the Z axis
    pub locked_axes: LockedAxes,
    /// [`Transform`]
    pub transform: Transform,
    /// [`GlobalTransform`]
    pub global_transform: GlobalTransform,
    /// [`Alliegance`]
    pub alliegance: Alliegance,
    /// [`Inventory`]
    pub inventory: Inventory,
    /// Serializable builder for an [`Equipped`]
    pub equipped: EquippedBuilder,
    /// [`CollisionLayers`]
    pub collision_layers: CollisionLayers,
    /// Starting [`Credits`]
    pub credits: Credits,
    /// Initial physics [`Rotation`]
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
                filters: LayerMask::from([PhysicsCategory::Weapon]),
            },
            credits: default(),
            transform: Transform::default(),
            global_transform: GlobalTransform::IDENTITY,
            rotation: Rotation(Transform::default_z().rotation),
        }
    }
}
