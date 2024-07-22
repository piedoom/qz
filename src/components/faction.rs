//! Non players

use bevy::{prelude::*, utils::HashSet};
use rand::RngCore;

#[derive(Component, Clone, Reflect, Default)]
pub struct Alliegance {
    /// The faction of the entity
    pub faction: Faction,
    /// Allied factions of this entity
    pub allies: FactionSet,
    /// Enemy factions of this entity
    pub enemies: FactionSet,
}

/// A faction is a simple UID that can be registered
#[derive(PartialEq, Eq, Copy, Clone, Reflect, Hash, Default)]
pub struct Faction(u32);

impl Faction {
    pub fn new() -> Faction {
        Self(rand::thread_rng().next_u32())
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}

#[derive(PartialEq, Eq, Clone, Reflect, Default)]
pub struct FactionSet {
    data: HashSet<Faction>,
    /// Whether to treat this as including all potential factions. Overrides faction data.
    all: bool,
}

impl FactionSet {
    pub fn all() -> Self {
        Self {
            data: [].into(),
            all: true,
        }
    }

    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        Q: std::hash::Hash + bevy::utils::hashbrown::Equivalent<Faction>,
    {
        if self.all {
            true
        } else {
            self.data.contains(value)
        }
    }
}

impl<const N: usize> From<[Faction; N]> for FactionSet {
    fn from(value: [Faction; N]) -> Self {
        FactionSet {
            data: value.into(),
            all: false,
        }
    }
}
