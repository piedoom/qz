use bevy::prelude::*;

use crate::prelude::*;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, manage_loss_state)
            .observe(on_save)
            .observe(on_load);
    }
}

fn on_save(trigger: Trigger<trigger::Save>) {
    todo!()
}

fn on_load(trigger: Trigger<trigger::Load>) {
    todo!()
}

// If all players are destroyed, players will be sent to the first slice (0)
fn manage_loss_state(
    mut cmd: Commands,
    mut inventories: Query<&mut Inventory>,
    mut cursor: ResMut<WorldCursor>,
    in_slices: Query<(Entity, &Slice), Without<Player>>,
    not_destroyed: Query<Entity, (Without<Destroyed>, With<Player>)>,
    destroyed: Query<Entity, (Added<Destroyed>, With<Player>)>,
) {
    // If all players are destroyed
    if !destroyed.is_empty() && not_destroyed.is_empty() {
        // Set every player to slice 0 and use inserts because its simpler than mutating here lol xd
        for entity in destroyed.iter() {
            cmd.entity(entity)
                .remove::<Destroyed>()
                .insert(Damage::default())
                .insert(Slice(0));
            let mut inv = inventories.get_mut(entity).unwrap(); // All players should have an inventory
            inv.drain();
        }

        // Despawn everything else and reset the world cursor
        for (entity, _) in in_slices.iter().filter(|(_, s)| s.0 != 0) {
            cmd.entity(entity).despawn_recursive();
        }
        ***cursor = 1;
    }
}
