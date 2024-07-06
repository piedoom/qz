//! Definitions for the settings of the application

use bevy::{prelude::*, window::WindowMode};

/// The root settings asset (and resource) for fine-tuning within the
/// application.
#[derive(
    serde::Deserialize,
    serde::Serialize,
    Resource,
    bevy::asset::Asset,
    bevy::reflect::TypePath,
    Clone,
)]
pub struct Settings {
    /// Application window related settings
    pub window: Window,
    /// Controls
    pub controls: Controls,
}

/// Application window related settings
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Window {
    /// The desired [`WindowMode`]
    pub mode: WindowMode,
}

/// Game controls
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Controls {
    /// Keyboard and mouse controls
    pub keyboard: KeyboardControls,
}

/// Keyboard controls
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct KeyboardControls {
    /// Turn left
    pub left: KeyCode,
    /// Turn right
    pub right: KeyCode,
    /// Thrust
    pub thrust: KeyCode,
    /// Brake
    pub brake: KeyCode,
}
