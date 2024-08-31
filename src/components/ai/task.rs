pub trait TaskLabel {
    const NAME: &'static str;
}

use super::*;

/// Seek out a new target to persue
#[derive(Component, Default)]
pub struct Search;
impl TaskLabel for Search {
    const NAME: &'static str = "search";
}

/// Close distance to target
#[derive(Component, Default)]
pub struct Persue;
impl TaskLabel for Persue {
    const NAME: &'static str = "persue";
}

/// Follow target and attack
#[derive(Component, Default)]
pub struct Attack;
impl TaskLabel for Attack {
    const NAME: &'static str = "attack";
}
