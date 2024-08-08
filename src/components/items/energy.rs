use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::EnergyError;

/// Item that contributes to a parent entity's [`Energy`] store
#[derive(Clone, Debug, Component, Reflect, Serialize, Deserialize, Default, PartialEq)]
pub struct Generator {
    /// How much [`Energy`] is restored per second
    pub recharge_rate: f32,
}

/// Item that contributes to the maximum storable [`Energy`] charge
#[derive(Clone, Debug, Component, Reflect, Serialize, Deserialize, Default, PartialEq)]
pub struct Battery(f32);

impl Battery {
    /// Creates a [`Battery`] with a specific capacity
    pub fn from_capacity(capacity: f32) -> Self {
        Self(capacity)
    }
    /// Gets the battery capacity
    pub fn capacity(&self) -> f32 {
        self.0
    }
}

/// Available energy attached to the top level entity. Things (such as [`Generator`]s) can
/// contribute to the `Energy`, which is consumed by other things like [`Weapon`]s.
#[derive(Clone, Debug, Component, Reflect, Serialize, Deserialize, Default)]
pub struct Energy(f32);

impl Energy {
    /// Attempt to consume an amount of energy. Errors if insufficient energy
    pub fn consume(&mut self, amount: f32) -> Result<(), EnergyError> {
        let new_charge = self.0 - amount;
        match new_charge.is_sign_negative() {
            true => Err(EnergyError::InsufficientCharge {
                requested: amount,
                actual: self.0,
            }),
            false => {
                self.0 = new_charge;
                Ok(())
            }
        }
    }

    /// Clamp the energy to a specified maximum bound
    #[inline(always)]
    pub fn clamp(&mut self, max: f32) {
        self.0 = self.0.min(max);
    }

    /// Obtain the current charge
    #[inline(always)]
    pub fn charge(&self) -> f32 {
        self.0
    }
}

impl std::ops::Add for Energy {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for Energy {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl From<f32> for Energy {
    fn from(value: f32) -> Self {
        Self(value)
    }
}
