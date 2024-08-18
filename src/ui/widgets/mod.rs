mod bar;
mod map;

use bevy::ecs::system::SystemParam;
use bevy_egui::egui::Widget;
pub use {bar::*, map::*};

/// Widget that takes in a query
pub trait QueryWidget: Widget {
    /// Query for this widget
    type Query: SystemParam;
}
