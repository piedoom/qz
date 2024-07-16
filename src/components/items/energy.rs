use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Component, Reflect, Serialize, Deserialize, Default)]
pub struct Energy {
    pub capacity: usize,
    /// TODO: decouple available energy from equipped energy
    #[serde(default)]
    pub charge: f32,
    pub recharge_rate: f32,
}

impl Energy {
    pub fn consume(&mut self, amount: f32) -> Result<(), EnergyError> {
        let new_charge = self.charge - amount;
        match new_charge.is_sign_negative() {
            true => Err(EnergyError::InsufficientCharge {
                requested: amount,
                actual: self.charge,
            }),
            false => {
                self.charge = new_charge;
                Ok(())
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum EnergyError {
    #[error("requested `{requested}` energy when only `{actual}` is available")]
    InsufficientCharge { requested: f32, actual: f32 },
}
