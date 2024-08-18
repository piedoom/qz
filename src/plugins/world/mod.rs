mod building;
mod creature;
mod gate;
mod zone;

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
use creature::*;
use gate::*;
use leafwing_input_manager::InputManagerBundle;
use petgraph::graph::NodeIndex;
use rand::{seq::*, Rng};
use zone::*;

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
            .register_type::<components::Gate>()
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
            .init_resource::<Universe>()
            .add_systems(OnEnter(AppState::New), (setup, generate))
            .add_systems(OnExit(AppState::load_game()), setup)
            .add_systems(
                Update,
                (
                    manage_spawners,
                    manage_gates,
                    setup_health,
                    cleanup_empty_chests,
                    update_background_shaders,
                    add_models,
                )
                    .run_if(in_state(AppState::main())),
            )
            .observe(on_spawn_creature)
            .observe(on_spawn_gate)
            .observe(on_spawn_building)
            .observe(on_spawn_zone);
    }
}

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    crafts: Res<Assets<Craft>>,
    library: Res<Library>,
    mut factions: ResMut<Factions>,
    assets: Res<AssetServer>,
) {
    let player_faction = factions
        .get_faction("player")
        .cloned()
        .unwrap_or_else(|| factions.register("player"));
    let enemy_faction = factions
        .get_faction("enemy")
        .cloned()
        .unwrap_or_else(|| factions.register("enemy"));
    let player_alliegance = Alliegance {
        faction: player_faction,
        allies: [player_faction].into(),
        enemies: [enemy_faction].into(),
    };
    // cmd.spawn(bevy::pbr::FogVolumeBundle {
    //     transform: Transform::from_scale(Vec3::splat(35.0)),
    //     ..default()
    // });

    // Spawn player
    cmd.spawn((
        Player(0), // TODO: handle IDs for multiplayer
        Name::new("player"),
        InputManagerBundle::<crate::prelude::Action>::default(),
        ChestsInRange {
            chests: default(),
            range: 5f32,
        },
        DockInRange {
            dock: None,
            range: 5f32,
        },
        CraftBundle {
            craft: crafts.get(&library.craft("bev").unwrap()).unwrap().clone(),
            alliegance: player_alliegance.clone(),
            inventory: Inventory::default(),
            equipped: EquippedBuilder {
                equipped: [
                    "minireactor.generator",
                    "light_laser.weapon",
                    "autoweld.repair",
                    "ion.battery",
                    "ion.battery",
                    "iron.armor",
                    "iron.armor",
                ]
                .map(ToString::to_string)
                .into(),
                slots: [
                    (EquipmentTypeId::Weapon, 1),
                    (EquipmentTypeId::RepairBot, 1),
                    (EquipmentTypeId::Generator, 1),
                    (EquipmentTypeId::Battery, 3),
                    (EquipmentTypeId::Armor, 3),
                ]
                .into(),
            },
            ..default()
        },
    ))
    .with_children(|cmd| {
        cmd.spawn(SceneBundle {
            scene: library.model("crafts/pest").unwrap(),
            ..default()
        });

        cmd.spawn((
            MaterialMeshBundle {
                transform: Transform::from_translation(Vec3::Y * -8f32),
                mesh: meshes.add(Plane3d::default().mesh().size(100.0, 100.0)),
                material: assets.add(BackgroundMaterial {
                    position: default(),
                }),
                ..default()
            },
            NotShadowCaster,
            NotShadowReceiver,
        ));
    });

    cmd.spawn((
        DirectionalLightBundle {
            transform: Transform::from_translation(Vec3::new(5f32, 5f32, 10f32))
                .looking_at(Vec3::ZERO, Vec3::Z),
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        },
        VolumetricLight,
    ));

    // Spawn camera
    cmd.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(0f32, -1f32, 16f32)
                .looking_at(Vec3::splat(0f32), Dir3::Z),
            ..default()
        },
        // VolumetricFogSettings {
        //     density: 0.05,
        //     absorption: 0.03,
        //     ..Default::default()
        // },
        BloomSettings::OLD_SCHOOL,
        // FogSettings {
        //     color: Color::srgb(0.25, 0.25, 0.25),
        //     falloff: FogFalloff::Linear {
        //         start: 5.0,
        //         end: 20.0,
        //     },
        //     ..default()
        // },
    ));
}

fn generate(
    mut cmd: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut universe: ResMut<Universe>,
    mut rng: ResMut<GlobalRng>,
    universe_position: Option<ResMut<UniversePosition>>,
) {
    // We're going to generate this new section without connecting any nodes, and then we will attach it
    let length = 4..=8;
    let nodes_per_layer = 1..=4;

    // Begin graph generation
    let length = rng.usize(length.clone());

    // We'll also save the first node
    let mut first_node: NodeIndex = default();

    // Contains all the nodes in the previous layer
    let mut previous_nodes = vec![];

    // I love LUA!
    for z in 1..=length {
        // If first or final, ensure only one node is spawned
        match z {
            0 => unreachable!(),
            // First layer
            1 => {
                // Set up the first node. There will always be a single node on the first layer
                first_node = universe.graph.add_node(Zone::from_depth(z - 1));
                previous_nodes = [first_node].into();
                // If the universe position doesn't exist, we insert it now
                if universe_position.is_none() {
                    cmd.insert_resource(UniversePosition::from(first_node));
                }
            }
            // Every other layer
            2.. => {
                // Spawn a random number of nodes on this layer. There must always be at least one
                let nodes_per_layer = rng.usize(nodes_per_layer.clone()).max(1);
                let nodes: Vec<_> = (0..nodes_per_layer)
                    .map(|_| {
                        // literally dont worry about the index stuff ok?
                        let node = universe.graph.add_node(Zone::from_depth(z - 1));

                        // Connect to a previous node at random
                        let prev_node = rng.sample(&previous_nodes).unwrap();
                        universe.graph.add_edge(*prev_node, node, ());
                        node
                    })
                    .collect();

                previous_nodes = nodes.clone();

                // If the last in the section...
                if z == length {
                    // Connect our newly generated section onto the existing universe endpoints.
                    // If no other sections exist yet, this won't do anything
                    for previous_end in universe.end.clone().iter() {
                        universe.graph.add_edge(*previous_end, first_node, ());
                    }

                    // We're all connected!

                    // Update the end of this universe
                    universe.end = nodes;
                }
            }
        }
    }

    next_state.set(AppState::LoadZone {
        load: first_node,
        previous: None,
    });
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
                    cmd.trigger(trigger::SpawnCreature {
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

fn manage_gates(
    mut next_state: ResMut<NextState<AppState>>,
    player_actions: Query<
        &leafwing_input_manager::action_state::ActionState<crate::prelude::Action>,
        With<Player>,
    >,
    gates: Query<(&Gate, &CollidingEntities)>,
) {
    for (gate, collisions) in gates.iter() {
        for collision in collisions.iter() {
            if let Ok(actions) = player_actions.get(*collision) {
                if actions.just_pressed(&crate::prelude::Action::Interact) {
                    // TODO move this logic to states
                    next_state.set(AppState::TransitionZone {
                        load: gate.destination(),
                    })
                }
            }
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
                        if x.1.path() == Some(&model.0) {
                            Some(x.1)
                        } else {
                            None
                        }
                    })
                    .unwrap()
                    .clone(),
                ..Default::default()
            },));
        });
    }
}
