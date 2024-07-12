use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Items to drop upon destruction
#[derive(Debug, Clone, Component, Reflect, Deserialize, Serialize, Deref, DerefMut)]
pub struct Drops(
    /// Items to drop mapped to a range of amount to drop normalized value determining drop rate
    pub HashMap<Item, DropRate>,
);

#[derive(Debug, Clone, Reflect, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct DropRate {
    pub min: usize,
    pub max: usize,
    /// 1 in X chance to drop. 1 will always drop.
    pub d: usize,
}
