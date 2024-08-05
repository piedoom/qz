//! Game library

#![feature(result_flattening)]
#![feature(assert_matches)]
#![allow(incomplete_features)]
#![feature(inherent_associated_types)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
// #![deny(missing_docs)]

mod components;
mod error;
pub(crate) mod plugins;
mod resources;
mod states;
mod util;

/// Contains the most common types for our application
pub mod prelude {
    use super::*;
    #[allow(unused_imports)]
    pub use {components::*, error::*, plugins::*, resources::*, states::*, util::*};
}
