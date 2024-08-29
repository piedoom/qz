use crate::prelude::*;
use bevy::prelude::*;
use bevy_turborand::prelude::*;
use std::f32::consts::TAU;

// /// Deserialize and build a zone from a [`ZoneDescription`]
// pub(super) fn on_load_zone()

/// Generate a zone
pub(super) fn on_generate_chunk(
    trigger: Trigger<triggers::GenerateChunks>,
    mut chunks: ResMut<Chunks>,
    mut cmd: Commands,
    mut rng: ResMut<GlobalRng>,
    factions: Res<Factions>,
) {
    let triggers::GenerateChunks { chunk_indicies } = trigger.event();

    let player_faction = factions.get_faction("player").unwrap();
    let enemy_faction = factions.get_faction("enemy").unwrap();

    for chunk_index in chunk_indicies {
        // If the chunk is not already generated...
        if !chunks.is_generated(chunk_index) {
            // Generate the chunk within a parent with offset determined by the chunk
            let rand_point = |rng: &mut GlobalRng| -> Vec2 {
                let mut t = Transform::default_z();
                t.rotate_z(rng.f32() * TAU);
                let point = t.forward() * 10f32;
                // Add chunk offset and return
                point.truncate() + chunk_index.to_world_coordinates()
            };

            let mut rotation = Transform::default_z();
            rotation.rotate_z(rng.f32() * TAU);

            cmd.trigger(triggers::SpawnBuilding {
                name: "nest".into(),
                translation: rand_point(&mut rng),
                rotation: 0f32,
                alliegance: Alliegance {
                    faction: *enemy_faction,
                    allies: [*enemy_faction].into(),
                    enemies: [*player_faction].into(),
                },
            });

            cmd.trigger(triggers::SpawnBuilding {
                name: "store".into(),
                translation: rand_point(&mut rng),
                rotation: 0f32,
                alliegance: Alliegance {
                    faction: *player_faction,
                    allies: [*player_faction].into(),
                    enemies: [*enemy_faction].into(),
                },
            });

            // Add the chunk to loaded chunks
            chunks.insert(*chunk_index);
        }
    }
}
