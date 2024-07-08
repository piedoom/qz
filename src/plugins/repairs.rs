//! Shields, basically
use bevy::prelude::*;

use crate::prelude::*;

pub struct RepairsPlugin;
impl Plugin for RepairsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_repairs);
    }
}

fn handle_repairs(
    repairs: Query<(&RepairBot, &Parent)>,
    mut damages: Query<&mut Damage, Without<Destroyed>>,
    time: Res<Time>,
) {
    for (repair, parent) in repairs.iter() {
        // Get parent damage
        if let Ok(mut damage) = damages.get_mut(parent.get()) {
            if **damage != 0f32 {
                // multiply the repair rate by our delta time
                let new_damage = (**damage - (repair.rate * time.delta_seconds())).max(0f32); // It's OK if we go over damage as it'll just destroy the entity
                **damage = new_damage;
            }
        }
    }
}
