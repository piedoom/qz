//! [`Action`]s to be used with `big_brain`

use bevy::prelude::*;
use big_brain::prelude::*;

/// If [`Weapon`]s are [`Equipped`], set them to ready and begin firing
#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct Attack;

/// Move towards a [`Targeting`] entity or position stored as a [`Waypoint`]
#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct Persue;
