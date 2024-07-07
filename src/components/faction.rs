//! Non players

use avian3d::prelude::PhysicsLayer;
use bevy::prelude::*;

#[derive(Component, Copy, Clone)]
pub struct Alliegance {
    /// The faction of the entity
    pub faction: Faction,
    /// Allied factions of this entity
    pub allies: Faction,
    /// Enemy factions of this entity
    pub enemies: Faction,
}

bitflags::bitflags! {
    #[derive(PartialEq, Eq, Copy, Clone)]
    pub struct Faction: u32 {
        const PLAYER = 1;
        const ENEMY = 2;
    }
}
impl PhysicsLayer for Faction {
    fn to_bits(&self) -> u32 {
        self.bits()
    }

    fn all_bits() -> u32 {
        Self::all().bits()
    }
}
