use crate::prelude::*;
use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};

/// All registered factions in the game. Factions can be registered and given an ID, which are then referenced with this resource.
#[derive(Resource, Default, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct Factions {
    by_name: HashMap<String, Faction>,
    by_id: HashMap<Faction, String>,
}

impl Factions {
    /// Register a name and return the faction, or return the faction of any existing faction with the given name
    pub fn register_or_retrieve(&mut self, name: impl AsRef<str>) -> Faction {
        let maybe_faction_id = self.get_faction(name.as_ref());
        match maybe_faction_id {
            Some(faction) => *faction,
            None => self.register(name.as_ref()),
        }
    }
    /// Register a name and return a given ID that will represent that faction. If a faction with this name already exists,
    /// it will be overwritten and replaced with a new ID
    pub fn register(&mut self, name: impl Into<String>) -> Faction {
        let faction = Faction::new();
        self.insert(name, faction);
        faction
    }
    pub fn insert(&mut self, name: impl Into<String>, id: impl Into<Faction>) {
        let name = name.into();
        let id = id.into();
        self.by_name.insert(name.clone(), id);
        self.by_id.insert(id, name);
    }
    pub fn get_faction_name(&self, id: impl AsRef<Faction>) -> Option<&String> {
        self.by_id.get(id.as_ref())
    }
    pub fn get_faction(&self, name: impl AsRef<str>) -> Option<&Faction> {
        self.by_name.get(name.as_ref())
    }
}
