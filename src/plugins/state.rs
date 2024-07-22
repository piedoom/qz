use bevy::prelude::*;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, manage_events);
    }
}

fn manage_events(mut events: EventReader<StateEvent>) {
    for event in events.read() {
        match event {
            StateEvent::Save => {
                //
            }
            StateEvent::Load => {
                //
            }
        }
    }
}

#[derive(Event)]
pub enum StateEvent {
    Save,
    Load,
}
