//! [`Scorer`]s to be used with `big_brain`

use bevy::prelude::*;
use big_brain::prelude::*;

/// 1 = facing target
#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct Facing;
/// Scorer for the normalized energy available to an entity
#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct Energy;
