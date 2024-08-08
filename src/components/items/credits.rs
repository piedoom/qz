use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
/// Total credits belonging to an entity
#[derive(Component, Reflect, Serialize, Deserialize, Default, Clone, Copy, Debug)]
pub struct Credits(usize);

impl Credits {
    /// Gets the credits value
    pub fn get(&self) -> usize {
        self.0
    }
    /// Create a new `Credits` component with `amount` credits
    pub fn new(amount: usize) -> Self {
        Self(amount)
    }
    /// Transfer from this store to another `Credits`. Errors if insufficient funds.
    pub fn transfer(&mut self, other: &mut Self, amount: usize) -> Result<(), CreditsError> {
        if self.0 < amount {
            Err(CreditsError::InsufficientCredits)
        } else {
            self.0 -= amount;
            other.0 += amount;
            Ok(())
        }
    }
}

/// Errors for `Credits` transations
#[derive(Error, Debug)]
pub enum CreditsError {
    /// Insufficient credits
    #[error("insufficient credits")]
    InsufficientCredits,
}
