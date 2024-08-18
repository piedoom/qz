use crate::prelude::*;
use bevy::prelude::*;
use bevy_turborand::prelude::*;
use petgraph::visit::EdgeRef;
use std::f32::consts::TAU;

// /// Deserialize and build a zone from a [`ZoneDescription`]
// pub(super) fn on_load_zone()

/// Generate a zone
pub(super) fn on_spawn_zone(
    trigger: Trigger<trigger::SpawnZone>,
    mut cmd: Commands,
    mut rng: ResMut<GlobalRng>,
    universe: Res<Universe>,
    factions: Res<Factions>,
) {
    let trigger::SpawnZone { node } = trigger.event();

    let player_faction = factions.get_faction("player").unwrap();
    let enemy_faction = factions.get_faction("enemy").unwrap();

    let rand_point = |rng: &mut GlobalRng| -> Vec2 {
        let mut t = Transform::default_z();
        t.rotate_z(rng.f32() * TAU);
        let point = t.forward() * 10f32;
        point.truncate()
    };

    let mut rotation = Transform::default_z();
    rotation.rotate_z(rng.f32() * TAU);

    // Find necessary gates to spawn
    let endpoints = universe
        .graph
        .edges(*node)
        .map(|edge| universe.graph.edge_endpoints(edge.id()).unwrap())
        .collect::<Vec<_>>();
    let endpoints_len = endpoints.len();

    let gates = endpoints
        .into_iter()
        .map(|(start, end)| {
            let destination = if start == *node { end } else { start };
            let t = trigger::SpawnGate {
                translation: (rotation.forward() * 10f32).truncate(),
                destination,
            };
            rotation.rotate_z(TAU / endpoints_len as f32);
            t
        })
        .collect::<Vec<_>>();

    cmd.trigger(trigger::SpawnBuilding {
        name: "nest".into(),
        translation: rand_point(&mut rng),
        rotation: 0f32,
        alliegance: Alliegance {
            faction: *enemy_faction,
            allies: [*enemy_faction].into(),
            enemies: [*player_faction].into(),
        },
    });

    for gate in gates {
        cmd.trigger(gate);
    }

    // Spawn stores at dead-ends
    if endpoints_len == 1 {
        cmd.trigger(trigger::SpawnBuilding {
            name: "store".into(),
            translation: rand_point(&mut rng),
            rotation: 0f32,
            alliegance: Alliegance {
                faction: *player_faction,
                allies: [*player_faction].into(),
                enemies: [*enemy_faction].into(),
            },
        });
    }
}
