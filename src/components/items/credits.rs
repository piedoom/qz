use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Component, Reflect, Serialize, Deserialize, Default)]
pub struct Credits(usize);

impl Credits {
    pub fn get(&self) -> usize {
        self.0
    }
    pub fn new(amount: usize) -> Self {
        Self(amount)
    }
    pub fn transfer(&mut self, other: &mut Self, amount: usize) -> Result<(), CreditsError> {
        if self.0 < amount {
            return Err(CreditsError::InsufficientCredits);
        } else {
            self.0 -= amount;
            other.0 += amount;
            Ok(())
        }
    }
}

#[derive(Error, Debug)]
pub enum CreditsError {
    #[error("insufficient credits")]
    InsufficientCredits,
}
