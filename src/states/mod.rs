use std::path::PathBuf;

use bevy::{asset::AssetPath, prelude::*};

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

    /// Create a new game and generate the world, additionally assigning a save game name
    NewGame,

    /// Save the current universe
    SaveGame {
        save_path: PathBuf,
    },
    /// Load a game
    LoadGame {
        path: PathBuf,
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
        Self::LoadGame { path: default() }
    }
    /// AppState::Main { .. }
    #[inline(always)]
    pub fn main() -> Self {
        Self::Main
    }
    /// AppState::SaveGame { .. }
    #[inline(always)]
    pub fn save_game() -> Self {
        Self::SaveGame {
            save_path: default(),
        }
    }
    /// AppState::NewGame { .. }
    #[inline(always)]
    pub fn new_game() -> Self {
        Self::NewGame
    }
}
