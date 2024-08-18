use bevy::{prelude::*, utils::HashSet};
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// Marks entities as allies or enemies of one another
#[derive(Component, Clone, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Alliegance {
    /// The faction of the entity
    pub faction: Faction,
    /// Allied factions of this entity
    pub allies: FactionSet,
    /// Enemy factions of this entity
    pub enemies: FactionSet,
}

/// A faction is a simple UID that can be registered
#[derive(PartialEq, Eq, Copy, Clone, Reflect, Hash, Default, Serialize, Deserialize)]
pub struct Faction(u32);

impl Faction {
    /// Create a new random faction ID
    pub fn new() -> Faction {
        Self(rand::thread_rng().next_u32())
    }

    /// Create a new faction without any explicitly assigned faction. It may, however, still resolve.
    pub fn none() -> Faction {
        Self::default()
    }

    /// Return this Faction ID
    pub fn id(&self) -> u32 {
        self.0
    }
}

/// A collection of `Faction`s
#[derive(PartialEq, Eq, Clone, Reflect, Default, Serialize, Deserialize)]
pub struct FactionSet {
    /// Hash set of each faction that is a part of this faction set. The `FactionSet` is essentially a `HashSet`
    /// except with the ability to set `all` without needing to specify every faction ID
    data: HashSet<Faction>,
    /// Whether to treat this as including all potential factions. Overrides faction data.
    all: bool,
}

impl FactionSet {
    /// Create a new `FactionSet` including all factions
    pub fn all() -> Self {
        Self {
            data: [].into(),
            all: true,
        }
    }

    /// Returns `true` if the set contains a value.
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
