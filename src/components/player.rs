//! Player marker

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Player marker
#[derive(Debug, Clone, Copy, Component, Reflect, Serialize, Deserialize, DerefMut, Deref)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Player(pub usize);
