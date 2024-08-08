use std::time::Duration;

use avian3d::{math::TAU, prelude::*};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_turborand::prelude::*;
use big_brain::prelude::*;
use leafwing_input_manager::prelude::*;

use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use rand::seq::SliceRandom;
use rand::Rng;
use trigger::SpawnCreature;

use crate::prelude::*;

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
            .register_type::<components::Equipped>()
            .register_type::<components::EquipmentType>()
            .register_type::<components::Faction>()
            .register_type::<components::Gate>()
            .register_type::<components::Health>()
            .register_type::<components::InRange>()
            .register_type::<components::Inventory>()
            .register_type::<components::Item>()
            .register_type::<components::Lifetime>()
            .register_type::<components::Npc>()
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
            .insert_resource(AmbientLight::NONE)
            .init_resource::<Universe>()
            .add_systems(OnEnter(AppState::main()), setup)
            .add_systems(
                Update,
                (
                    manage_spawners,
                    manage_gates,
                    setup_health,
                    cleanup_empty_chests,
                )
                    .run_if(in_state(AppState::main())),
            )
            .observe(on_spawn_creature)
            .observe(on_spawn_gate)
            .observe(on_spawn_building)
            .observe(on_despawn_zone)
            .observe(on_generate_section)
            .observe(on_generate_zone);
    }
}

fn setup(
    mut cmd: Commands,
    mut factions: ResMut<Factions>,
    mut load_events: EventWriter<events::Load>,
    library: Res<Library>,
    crafts: Res<Assets<Craft>>,
) {
    let player_faction = factions.register("player");
    let enemy_faction = factions.register("enemy");
    let player_alliegance = Alliegance {
        faction: player_faction,
        allies: [player_faction].into(),
        enemies: [enemy_faction].into(),
    };

    // Spawn player
    cmd.spawn((
        Player(0), // TODO: handle IDs for multiplayer
        Name::new("Player"),
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
                    "dart_2.weapon",
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
    ));

    // Spawn camera
    cmd.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0f32, -2f32, 36.0).looking_at(Vec3::ZERO, Dir3::Z),
            ..default()
        },
        FogSettings {
            color: Color::srgb(0.25, 0.25, 0.25),
            falloff: FogFalloff::Linear {
                start: 5.0,
                end: 20.0,
            },
            ..default()
        },
    ));

    cmd.trigger(trigger::GenerateSection {
        length: 5..=7,
        nodes_per_layer: 1..=2,
    });

    load_events.send(events::Load {
        node: None,
        from_node: None,
    });
}

fn on_spawn_creature(
    trigger: Trigger<trigger::SpawnCreature>,
    mut cmd: Commands,
    mut rng: ResMut<GlobalRng>,
    library: Res<Library>,
    creatures: Res<Assets<Creature>>,
    crafts: Res<Assets<Craft>>,
    items: Res<Assets<Item>>,
) {
    let SpawnCreature {
        name,
        translation,
        rotation,
        alliegance,
        spawner,
    } = trigger.event();
    let creature = library.creature(name).unwrap();
    let Creature {
        name,
        craft,
        drops,
        inventory,
        equipped,
        range,
        credits,
    } = creatures.get(&creature).cloned().unwrap();
    let craft = library
        .crafts
        .get(&format!("crafts/{}.craft.ron", craft))
        .and_then(|craft| crafts.get(craft))
        .unwrap();
    let drops = drops
        .into_iter()
        .filter_map(|(drop_name, drop_rate)| {
            library
                .items
                .get(&format!("items/{}.ron", drop_name))
                .map(|item| (item.clone(), drop_rate))
        })
        .collect();
    let mut ent = cmd.spawn((
        CraftBundle {
            collider: Collider::sphere(craft.size * 0.5),
            mass: Mass(craft.mass),
            craft: craft.clone(),
            transform: Transform::z_from_parts(translation, rotation),
            alliegance: alliegance.clone(),
            inventory: Inventory::with_capacity(craft.capacity)
                .with_many_from_str(
                    inventory.into_iter().collect::<HashMap<String, usize>>(),
                    &items,
                    &library,
                )
                .unwrap(),
            equipped,
            ..default()
        },
        Npc,
        Drops(drops),
        InRange::new(range),
        Name::new(name),
    ));
    if let Some(spawner) = spawner {
        ent.insert((SpawnedFrom(*spawner),));
    }

    let credits = rng.usize(credits.0..=credits.1);
    if credits != 0 {
        ent.insert(Credits::new(credits));
    }

    ent.insert(
        Thinker::build()
            .picker(FirstToScore { threshold: 0.8 })
            .when(
                scorers::Facing,
                Concurrently::build()
                    .push(actions::Attack)
                    .push(actions::Persue),
            )
            .when(
                EvaluatingScorer::build(scorers::Facing, LinearEvaluator::new_inversed()),
                actions::Persue,
            ),
        // .when(
        //     scorers::Danger {
        //         radius: 3f32..=15f32,
        //     },
        //     actions::Retreat,
        // ),
    );
}

fn on_spawn_gate(trigger: Trigger<trigger::SpawnGate>, mut cmd: Commands) {
    const GATE_RADIUS: f32 = 2.0f32;
    let trigger::SpawnGate {
        translation,
        destination,
    } = trigger.event();

    cmd.spawn((
        Structure,
        Sensor,
        Collider::sphere(GATE_RADIUS),
        CollisionLayers {
            memberships: LayerMask::ALL,
            filters: LayerMask::ALL,
        },
        Gate::new(*destination),
        Transform::z_from_parts(translation, &0f32),
    ));
}

fn on_spawn_building(
    trigger: Trigger<trigger::SpawnBuilding>,
    mut cmd: Commands,
    library: Res<Library>,
    buildings: Res<Assets<Building>>,
    items: Res<Assets<Item>>,
) {
    let trigger::SpawnBuilding {
        name,
        translation,
        rotation,
        alliegance,
    } = trigger.event();
    let Building {
        name,
        mass,
        health,
        size,
        drops,
        inventory,
        inventory_space,
        equipped,
        spawner,
        store,
        credits,
    } = library
        .building(name)
        .and_then(|building| buildings.get(building.id()))
        .unwrap()
        .clone();

    let mut entity = cmd.spawn((
        Name::new(name.clone()),
        Structure,
        Health::from(health),
        Damage::default(),
        RigidBody::Dynamic,
        Mass(mass),
        Collider::sphere(size * 0.5),
        alliegance.clone(),
        Inventory::with_capacity(inventory_space)
            .with_many_from_str(inventory.into_iter().collect(), &items, &library)
            .unwrap(),
        equipped,
        Drops(
            drops
                .into_iter()
                .filter_map(|(drop, rate)| library.item(drop).map(|x| (x, rate)))
                .collect(),
        ),
        CollisionLayers {
            memberships: LayerMask::from([PhysicsCategory::Structure]),
            filters: LayerMask::from([PhysicsCategory::Weapon, PhysicsCategory::Structure]),
        },
        LockedAxes::ROTATION_LOCKED,
        Transform::z_from_parts(translation, rotation),
    ));

    if let Some(spawner) = spawner {
        entity.insert((spawner,));
    }

    if let Some(credits) = credits {
        entity.insert(Credits::new(credits));
    }

    if let Some(store) = store {
        entity.insert((
            Store {
                items: store
                    .into_iter()
                    .map(|(n, o)| (library.item(n).unwrap(), o))
                    .collect(),
            },
            Dockings::default(),
        ));
    }
}

/// Generates a new map section in the graph from an optionally given node index.
fn on_generate_section(
    trigger: Trigger<trigger::GenerateSection>,
    mut cmd: Commands,
    mut universe: ResMut<Universe>,
    mut rng: ResMut<GlobalRng>,
    universe_position: Option<ResMut<UniversePosition>>,
) {
    let trigger::GenerateSection {
        length,
        nodes_per_layer,
    } = trigger.event();

    // We're going to generate this new section without connecting any nodes, and then we will attach it

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
                first_node = universe.graph.add_node(Zone::new(0));
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
                        let node = universe.graph.add_node(Zone::new(z - 1));
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
}

fn on_generate_zone(
    trigger: Trigger<trigger::GenerateZone>,
    mut universe: ResMut<Universe>,
    mut rng: ResMut<GlobalRng>,
    factions: Res<Factions>,
    assets: Res<AssetServer>,
) {
    let trigger::GenerateZone { node } = trigger.event();

    // Double check that:
    // 1. The node exists in our universe
    // 2. the zone doesn't already have a scene
    if universe
        .graph
        .node_weight(*node)
        .map(|zone| zone.scene.is_some())
        .unwrap_or(true)
    {
        panic!("node does not exist or zone already has a scene");
    }

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
            dbg!((start, end));
            let destination = if start == *node { end } else { start };
            let t = trigger::SpawnGate {
                translation: (rotation.forward() * 10f32).truncate(),
                destination,
            };
            rotation.rotate_z(TAU / endpoints_len as f32);
            t
        })
        .collect();

    let zd: ZoneDescription = ZoneDescription {
        buildings: [trigger::SpawnBuilding {
            name: "nest".into(),
            translation: rand_point(&mut rng),
            rotation: 0f32,
            alliegance: Alliegance {
                faction: *enemy_faction,
                allies: [*enemy_faction].into(),
                enemies: [*player_faction].into(),
            },
        }]
        .into(),
        gates,
    };

    // Set the scene into the node
    let zone = universe.graph.node_weight_mut(*node).unwrap();
    let scene_handle = assets.add(zd);

    zone.scene = Some(scene_handle);
}

fn on_despawn_zone(
    trigger: Trigger<trigger::DespawnZone>,
    mut cmd: Commands,
    things: Query<Entity, (With<Collider>, Without<Player>)>,
) {
    let _ev = trigger.event();
    for thing in things.iter() {
        cmd.entity(thing).despawn_recursive();
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
                    cmd.trigger(SpawnCreature {
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
    mut cmd: Commands,
    mut events: EventWriter<events::Load>,
    mut universe_position: ResMut<UniversePosition>,
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
                    let old_universe_position = universe_position.0;
                    universe_position.0 = gate.destination();
                    cmd.trigger(trigger::DespawnZone);
                    events.send(events::Load {
                        node: Some(gate.destination()),
                        from_node: Some(old_universe_position),
                    });
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
            .insert((Health(craft.health), Damage::default()));
    }
}

fn cleanup_empty_chests(
    mut cmd: Commands,
    changed_chests: Query<(Entity, &Inventory), (With<Chest>, Changed<Inventory>)>,
    changed_credit_chests: Query<(Entity, &Credits), (With<Chest>, Without<Inventory>)>,
) {
    for (entity, inventory) in changed_chests.iter() {
        if inventory.is_empty() {
            cmd.entity(entity).despawn();
        }
    }
    for (entity, credits) in changed_credit_chests.iter() {
        if credits.get() == 0 {
            cmd.entity(entity).despawn();
        }
    }
}
