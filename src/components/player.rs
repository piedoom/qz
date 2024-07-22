//! Player marker

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Player marker
#[derive(Debug, Clone, Copy, Component, Reflect, Serialize, Deserialize, DerefMut, Deref)]
pub struct Player(pub usize);

/// NPC marker
#[derive(Component, Reflect)]
pub struct Npc;
