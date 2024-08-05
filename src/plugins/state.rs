use bevy::prelude::*;

use crate::prelude::*;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<events::Load>().add_systems(
            Update,
            (
                on_load,
                // manage_loss_state
            ),
        );
    }
}

fn on_load(
    mut cmd: Commands,
    mut events: ParamSet<(EventReader<events::Load>, EventWriter<events::Load>)>,
    mut transforms: Query<&mut Transform>,
    player_entities: Query<Entity, With<Player>>,
    gate_entities: Query<(Entity, &Gate)>,
    universe_position: Option<Res<UniversePosition>>,
    descriptions: Res<Assets<ZoneDescription>>,
    universe: Res<Universe>,
) {
    let mut new_events = vec![];
    for events::Load { node, from_node } in events.p0().read() {
        // use the universe point if no index is specified
        let node_index = node.unwrap_or(universe_position.as_ref().unwrap().get());
        if let Some(zone) = universe.graph.node_weight(node_index) {
            match &zone.scene {
                Some(scene) => {
                    // A scene for this zone is already saved, so we will spawn it
                    let ZoneDescription { buildings, gates } = descriptions.get(scene).unwrap();
                    for building_trigger in buildings {
                        cmd.trigger(building_trigger.clone());
                    }
                    for gate_trigger in gates {
                        cmd.trigger(gate_trigger.clone());
                    }
                    // Move the player to the newly spawned gate
                    if let Some(from_node) = from_node {
                        // Find the gate transform to copy
                        let gate = gate_entities
                            .iter()
                            .find(|(_, g)| g.destination() == *from_node);
                        if let Some((gate_entity, _)) = gate {
                            for player_entity in player_entities.iter() {
                                let gate_transform = *transforms.get(gate_entity).unwrap();
                                let mut player_transform =
                                    transforms.get_mut(player_entity).unwrap();
                                *player_transform = gate_transform;
                            }
                        }
                    }
                }
                None => {
                    // We must first generate a scene. We'll then run this again to spawn what we just generated
                    cmd.trigger(trigger::GenerateZone { node: node_index });
                    // Retry now that this zone must have an associated scene
                    new_events.push(events::Load {
                        node: *node,
                        from_node: *from_node,
                    });
                }
            }
        } else {
            panic!("attempted to load invalid node index {:?}", node_index);
        }
    }
    for event in new_events.into_iter() {
        events.p1().send(event);
    }
}

// // If all players are destroyed, players will be sent to the first slice (0)
// fn manage_loss_state(
//     mut cmd: Commands,
//     mut inventories: Query<&mut Inventory>,
//     mut cursor: ResMut<WorldCursor>,
//     in_slices: Query<(Entity, &Slice), Without<Player>>,
//     not_destroyed: Query<Entity, (Without<Destroyed>, With<Player>)>,
//     destroyed: Query<Entity, (Added<Destroyed>, With<Player>)>,
// ) {
//     // If all players are destroyed
//     if !destroyed.is_empty() && not_destroyed.is_empty() {
//         // Set every player to slice 0 and use inserts because its simpler than mutating here lol xd
//         for entity in destroyed.iter() {
//             cmd.entity(entity)
//                 .remove::<Destroyed>()
//                 .insert(Damage::default())
//                 .insert(Slice(0));
//             let mut inv = inventories.get_mut(entity).unwrap(); // All players should have an inventory
//             inv.drain();
//         }

//         // Despawn everything else and reset the world cursor
//         for (entity, _) in in_slices.iter().filter(|(_, s)| s.0 != 0) {
//             cmd.entity(entity).despawn_recursive();
//         }
//         ***cursor = 1;
//     }
// }
