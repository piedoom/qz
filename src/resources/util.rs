use bevy::prelude::*;

/// Determines whether to draw debug UI
#[derive(Default, Resource, Deref, DerefMut, PartialEq, Eq)]
pub struct DrawInspector(pub bool);

/// Name of the saved game
#[derive(Default, Resource, Deref, DerefMut, Clone)]
pub struct SaveGameName(pub String);

impl SaveGameName {
    pub fn new() -> Self {
        Self(
            (0..3)
                .map(|_| random_word::gen(random_word::Lang::En).to_string())
                .reduce(|acc, e| acc + " " + &e)
                .unwrap(),
        )
    }
}
