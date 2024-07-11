use std::ops::RangeInclusive;

use bevy::{prelude::*, utils::HashMap};

use crate::prelude::*;

/// Items to drop upon destruction
#[derive(Component, Reflect)]
pub struct Drop {
    /// Items to drop mapped to a range of amount to drop normalized value determining drop rate
    pub items: HashMap<Item, DropRate>,
}

#[derive(Reflect, PartialEq, Eq, Hash)]
pub struct DropRate {
    pub amount: RangeInclusive<usize>,
    /// 1 in X chance to drop. 1 will always drop.
    pub d: usize,
}
