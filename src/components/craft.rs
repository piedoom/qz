//! Moveable thing

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::prelude::*;

#[derive(Component, Reflect)]
pub struct Destroyed;

#[derive(Component, Reflect, Deref, DerefMut)]
pub struct Health(usize);

/// Damage inflicted. Used in tandem with [`Health`]. Damage is a float instead of an integer, as
/// repairs may repair fractional amounts
#[derive(Component, Reflect, Deref, DerefMut, Default)]
pub struct Damage(f32);

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
#[derive(Component, Reflect)]
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
    pub inventory: Inventory,
    pub equipment: Equipment,
    pub health: Health,
    pub damage: Damage,
    pub collision_layers: CollisionLayers,
    pub slice: Slice,
}

impl Default for CraftBundle {
    fn default() -> Self {
        Self {
            controller: Controller::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::sphere(0.5f32),
            mass: Mass(1f32),
            craft: Craft {
                speed: 10f32,
                rotation: 300f32,
                brake: 200f32,
                acceleration: 40f32,
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
            equipment: Equipment::default(),
            health: 64.into(),
            damage: default(),
            collision_layers: CollisionLayers {
                memberships: LayerMask::from([PhysicsCategory::Craft]),
                filters: LayerMask::from([PhysicsCategory::Craft, PhysicsCategory::Weapon]),
            },
            slice: default(),
        }
    }
}
