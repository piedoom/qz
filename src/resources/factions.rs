use crate::prelude::*;
use bevy::prelude::*;

/// All registered factions in the game. Factions can be registered and given an ID, which are then referenced with this resource.
#[derive(Resource, Default)]
pub struct Factions(bimap::BiHashMap<String, Faction>);

impl Factions {
    /// Register a name and return a given ID that will represent that faction. If a faction with this name already exists,
    /// it will be overwritten and replaced with a new ID
    pub fn register(&mut self, name: impl AsRef<str>) -> Faction {
        let faction = Faction::new();
        self.0.insert(name.as_ref().to_string(), faction);
        faction
    }

    /// Register a name and return the faction, or return the faction of any existing faction with the given name
    pub fn register_or_retrieve(&mut self, name: impl AsRef<str>) -> Faction {
        let maybe_faction = self.get_faction(name.as_ref());
        match maybe_faction {
            Some(faction) => *faction,
            None => self.register(name),
        }
    }

    /// Try to get the faction by its name
    pub fn get_faction(&self, name: impl AsRef<str>) -> Option<&Faction> {
        self.0.get_by_left(name.as_ref())
    }

    /// Try to get the faction name by the faction ID
    pub fn get_name(&self, faction: impl AsRef<Faction>) -> Option<&String> {
        self.0.get_by_right(faction.as_ref())
    }
}
