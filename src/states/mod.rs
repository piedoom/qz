use bevy::{asset::AssetPath, prelude::*};
use petgraph::graph::NodeIndex;

// use crate::prelude::UniverseDescription;

/// Controls the state of our application
#[derive(Default, States, Debug, Clone, Eq)]
pub enum AppState {
    /// The starting state of the application where all necessary files and
    /// assets are preloaded before moving on to the loading stage
    #[default]
    Preload,

    Menu,

    /// The main application state
    Main,

    /// New game
    New,

    /// Load an entire new universe
    LoadGame(AssetPath<'static>),

    /// Save the current universe
    SaveGame,

    TransitionZone {
        load: NodeIndex,
    },
    LoadZone {
        load: NodeIndex,
        previous: Option<NodeIndex>,
    },
}

// Discard data so we can use the gamestate to hold relevant information
impl PartialEq for AppState {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl std::hash::Hash for AppState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl AppState {
    /// AppState::Preloading { .. }
    #[inline(always)]
    pub fn preloading() -> Self {
        Self::Preload
    }

    #[inline(always)]
    pub fn menu() -> Self {
        Self::Menu
    }
    /// AppState::Loading { .. }
    #[inline(always)]
    pub fn load_game() -> Self {
        Self::LoadGame(Default::default())
    }
    /// AppState::Main { .. }
    #[inline(always)]
    pub fn main() -> Self {
        Self::Main
    }
    /// AppState::SaveGame { .. }
    #[inline(always)]
    pub fn save_game() -> Self {
        Self::SaveGame
    }
    /// Save the current zone and load the next
    #[inline(always)]
    pub fn transition_zone() -> Self {
        Self::TransitionZone { load: default() }
    }

    #[inline(always)]
    pub fn load_zone() -> Self {
        Self::LoadZone {
            load: default(),
            previous: default(),
        }
    }
}
