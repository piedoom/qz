//! Game library

#![feature(result_flattening)]
#![feature(assert_matches)]
#![allow(incomplete_features)]
#![feature(inherent_associated_types)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
// #![deny(missing_docs)]
// #![deny(clippy::missing_docs_in_private_items)]

/// All game components
mod components;
/// Game errors
mod error;
/// Plugin logic
pub(crate) mod plugins;
/// All game resources, such as input and factions
mod resources;
/// Game state
mod states;
/// UI widgets
pub mod ui;
/// Utility methods
mod util;

/// Contains the most common types for our application
pub mod prelude {
    use super::*;
    #[allow(unused_imports)]
    pub use {components::*, error::*, plugins::*, resources::*, states::*, ui::*, util::*};
}
