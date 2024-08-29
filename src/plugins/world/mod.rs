mod building;
mod chunk;
mod creature;

use std::{f32::consts::TAU, time::Duration};

use crate::prelude::*;
use avian3d::prelude::*;
use bevy::{
    core_pipeline::bloom::BloomSettings,
    pbr::{NotShadowCaster, NotShadowReceiver, VolumetricLight},
    prelude::*,
};
use bevy_turborand::prelude::*;
use building::*;
use chunk::*;
use creature::*;
use leafwing_input_manager::InputManagerBundle;
use petgraph::graph::NodeIndex;
use rand::{seq::*, Rng};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        use crate::components;
        app.register_type::<components::Alliegance>()
            .register_type::<components::Chest>()
            .register_type::<components::ChestsInRange>()
            .register_type::<components::Controller>()
            .register_type::<components::Craft>()
            .register_type::<components::Credits>()
            .register_type::<components::Damage>()
            .register_type::<components::Destroyed>()
            .register_type::<components::DockInRange>()
            .register_type::<components::Drops>()
            .register_type::<components::DropsBuilder>()
            .register_type::<components::DropRate>()
            .register_type::<components::Docked>()
            .register_type::<components::Dockings>()
            .register_type::<components::Generator>()
            .register_type::<components::Energy>()
            .register_type::<components::Equipped>()
            .register_type::<components::EquippedBuilder>()
            .register_type::<components::InventoryBuilder>()
            .register_type::<components::EquipmentType>()
            .register_type::<components::Faction>()
            .register_type::<Factions>()
            .register_type::<components::Health>()
            .register_type::<components::Heat>()
            .register_type::<components::InRange>()
            .register_type::<components::Inventory>()
            .register_type::<components::Item>()
            .register_type::<components::Lifetime>()
            .register_type::<components::Model>()
            .register_type::<components::Persistent>()
            .register_type::<components::Player>()
            .register_type::<components::Projectile>()
            .register_type::<components::RepairBot>()
            .register_type::<components::Store>()
            .register_type::<components::SpawnedFrom>()
            .register_type::<components::Spawner>()
            .register_type::<components::Structure>()
            .register_type::<components::Weapon>()
            .register_type::<components::WeaponType>()
            .insert_resource(ClearColor(Color::BLACK))
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 20.,
            })
            .add_systems(
                Update,
                (
                    manage_spawners,
                    setup_health,
                    cleanup_empty_chests,
                    update_background_shaders,
                    add_models,
                )
                    .run_if(in_state(AppState::main())),
            )
            .observe(on_spawn_creature)
            .observe(on_spawn_building)
            .observe(on_generate_chunk);
    }
}

fn manage_spawners(
    mut cmd: Commands,
    mut spawners: Query<(Entity, &mut Spawner, &Transform), Without<Destroyed>>,
    factions: Res<Factions>,
    spawned_from: Query<&SpawnedFrom, Without<Destroyed>>,
    time: Res<Time>,
) {
    let enemy_faction = *factions.get_faction("enemy").unwrap();
    let player_faction = *factions.get_faction("player").unwrap();
    for (entity, mut spawner, transform) in spawners.iter_mut() {
        let mut rng = rand::thread_rng();
        let new_time = spawner.last_tick + Duration::from_secs_f32(spawner.tick);
        if time.elapsed() >= new_time
            && spawned_from.iter().filter(|s| s.0 == entity).count() < spawner.maximum
        {
            // Go through our spawnlist and roll until we get a spawn
            let mut spawns = spawner.spawns.clone();
            // Shuffle potential spawns so we don't bias towards the first entries
            spawns.shuffle(&mut rng);
            for (spawn, d) in spawns.into_iter() {
                if rng.gen_ratio(1, d as u32) {
                    // Spawn thing
                    cmd.trigger(triggers::SpawnCreature {
                        name: spawn.clone(),
                        translation: transform.translation.truncate(),
                        rotation: rng.gen_range(0f32..TAU),
                        alliegance: Alliegance {
                            faction: enemy_faction,
                            allies: [enemy_faction].into(),
                            enemies: [player_faction].into(),
                        },
                        spawner: Some(entity),
                    });

                    break;
                }
            }

            spawner.last_tick = time.elapsed();
        }
    }
}

/// Health is sometimes determined on the object/item/craft,
/// so we can use this system to apply it
fn setup_health(mut cmd: Commands, crafts: Query<(Entity, &Craft), Added<Craft>>) {
    for (entity, craft) in crafts.iter() {
        cmd.entity(entity)
            .insert((Health::new(craft.health), Damage::default()));
    }
}

fn cleanup_empty_chests(
    mut cmd: Commands,
    changed_chests: Query<(Entity, &Inventory), (With<Chest>, Changed<Inventory>)>,
    changed_credit_chests: Query<(Entity, &Credits), (With<Chest>, Without<Inventory>)>,
) {
    for (entity, inventory) in changed_chests.iter() {
        if inventory.is_empty() {
            cmd.entity(entity).despawn_recursive();
        }
    }
    for (entity, credits) in changed_credit_chests.iter() {
        if credits.get() == 0 {
            cmd.entity(entity).despawn_recursive();
        }
    }
}

fn update_background_shaders(
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    camera: Query<&Transform, With<Camera>>,
) {
    if let Ok(camera_transform) = camera.get_single() {
        for (_, material) in materials.iter_mut() {
            material.position = camera_transform.translation.xy() * Vec2::new(1f32, -1f32);
        }
    }
}

fn add_models(
    mut cmd: Commands,
    entities: Query<(Entity, &Model), Added<Model>>,
    library: Res<Library>,
) {
    for (entity, model) in entities.iter() {
        cmd.entity(entity).with_children(|cmd| {
            cmd.spawn((SceneBundle {
                scene: library
                    .models
                    .iter()
                    .find_map(|x| {
                        if x.1.path() == Some(model.path()) {
                            Some(x.1)
                        } else {
                            None
                        }
                    })
                    .unwrap()
                    .clone(),
                transform: Transform::from_translation(model.offset()),
                ..Default::default()
            },));
        });
    }
}
