use bevy::prelude::*;
use big_brain::prelude::*;

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct Attack;

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct Disengage;

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct Persue;

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct Retreat;
