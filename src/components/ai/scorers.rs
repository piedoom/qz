use std::ops::RangeInclusive;

use bevy::prelude::*;
use big_brain::prelude::*;

use crate::util::RangeInclusiveExt;

#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct Danger {
    pub radius: RangeInclusive<f32>,
}

impl Danger {
    pub fn score(&self, distance_squared: f32) -> f32 {
        (self.radius.start().powi(2)..=self.radius.end().powi(2)).lerp(distance_squared)
    }
}

/// 1 = facing target
#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct Facing;

#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct Energy;

// TODO
#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct DistanceFromSpawn;
