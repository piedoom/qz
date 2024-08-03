use std::time::Duration;

use avian3d::{math::TAU, prelude::*};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_turborand::prelude::*;
use big_brain::prelude::*;
use leafwing_input_manager::prelude::*;
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
            .register_type::<components::Slice>()
            .register_type::<components::Store>()
            .register_type::<components::SpawnedFrom>()
            .register_type::<components::Spawner>()
            .register_type::<components::Structure>()
            .register_type::<components::Weapon>()
            .register_type::<components::WeaponType>()
            .insert_resource(ClearColor(Color::BLACK))
            .insert_resource(AmbientLight::NONE)
            .init_resource::<WorldCursor>()
            .init_resource::<DepthCursor>()
            .init_resource::<Universe>()
            .add_systems(OnEnter(AppState::main()), setup)
            .add_systems(
                Update,
                (
                    manage_spawners,
                    manage_slice_transforms.after(manage_gates),
                    manage_gates,
                    setup_health,
                    cleanup_empty_chests,
                )
                    .run_if(in_state(AppState::main())),
            )
            .observe(on_spawn_creature)
            .observe(on_spawn_gate)
            .observe(on_spawn_building)
            .observe(on_spawn_slice);
    }
}

fn setup(
    mut cmd: Commands,
    mut factions: ResMut<Factions>,
    mut universe: ResMut<Universe>,
    library: Res<Library>,
    crafts: Res<Assets<Craft>>,
) {
    let slice = cmd.spawn((Slice(0), Transform::default())).id();
    universe.add_node(slice);

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
            craft: crafts.get(&library.craft("pest").unwrap()).unwrap().clone(),
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

    cmd.trigger(trigger::SpawnSlice {
        slice: Slice(0),
        from_gate: None,
    });
}

fn on_spawn_creature(
    trigger: Trigger<trigger::SpawnCreature>,

    mut cmd: Commands,
    library: Res<Library>,
    creatures: Res<Assets<Creature>>,
    crafts: Res<Assets<Craft>>,
    items: Res<Assets<Item>>,
) {
    let SpawnCreature {
        name,
        slice,
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
            transform: Transform::z_from_parts(translation, rotation, slice),
            alliegance: alliegance.clone(),
            inventory: Inventory::with_capacity(craft.capacity)
                .with_many_from_str(
                    inventory.into_iter().collect::<HashMap<String, usize>>(),
                    &items,
                    &library,
                )
                .unwrap(),
            slice: *slice,
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
        use_entity,
        slice,
        translation,
        end_gate,
    } = trigger.event();
    let entity = use_entity.unwrap_or_else(|| cmd.spawn(()).id());

    cmd.entity(entity).insert((
        Structure,
        Sensor,
        Collider::sphere(GATE_RADIUS),
        CollisionLayers {
            memberships: LayerMask::ALL,
            filters: LayerMask::ALL,
        },
        *slice,
        Gate::new(*end_gate),
        Transform::z_from_parts(translation, &0f32, slice),
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
        slice,
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
        *slice,
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
        Transform::z_from_parts(translation, rotation, slice),
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

fn on_spawn_slice(
    trigger: Trigger<trigger::SpawnSlice>,
    mut cmd: Commands,
    mut rng: ResMut<GlobalRng>,
    mut gates: Query<&mut Gate>,
    transforms: Query<&Transform>,
    factions: Res<Factions>,
) {
    let trigger::SpawnSlice { slice, from_gate } = trigger.event();
    // Show a background grid
    cmd.spawn((
        Grid,
        *slice,
        Transform::z_from_parts(&Vec2::ZERO, &0f32, slice),
    ));

    let player_faction = *factions.get_faction("player").unwrap();
    let enemy_faction = *factions.get_faction("enemy").unwrap();

    // Rotate in a random direction and cast outwards
    let rand_point = |rng: &mut GlobalRng| -> Vec2 {
        let mut t = Transform::default_z();
        t.rotate_z(rng.f32() * TAU);
        let point = t.forward() * SEPARATION_SCALAR;
        point.truncate()
    };

    if **slice == 0 {
        // always spawn a store on layer 0
        cmd.trigger(trigger::SpawnBuilding {
            name: "store".to_string(),
            slice: *slice,
            translation: rand_point(&mut rng),
            rotation: 0f32,
            alliegance: Alliegance {
                faction: Faction::none(),
                allies: FactionSet::all(),
                enemies: [enemy_faction].into(),
            },
        });
    } else {
        let home_gate_chance = rng.chance(1f64 / 8f64);
        if home_gate_chance {
            cmd.trigger(trigger::SpawnGate {
                use_entity: None,
                slice: *slice,
                translation: rand_point(&mut rng),
                end_gate: None,
            });
        }
    }

    // If a from gate is set, solve loose ends
    if let Some(from_gate_entity) = from_gate {
        let from_gate_transform = transforms.get(*from_gate_entity).unwrap();

        // Entity will be created in next layer
        let return_gate_entity = cmd.spawn(()).id();

        // Spawn the gate back
        cmd.trigger(trigger::SpawnGate {
            slice: *slice,
            translation: from_gate_transform.translation.truncate(),
            end_gate: Some(*from_gate_entity),
            use_entity: Some(return_gate_entity),
        });

        // Set the gate to the new spawned layer
        let mut from_gate = gates.get_mut(*from_gate_entity).unwrap();
        from_gate.0 = Some(return_gate_entity);
    }

    // TODO: Graph stuff and ending gates

    const SEPARATION_SCALAR: f32 = 24.0;
    let store_chance = rng.chance(1f64 / 3f64);

    cmd.trigger(trigger::SpawnGate {
        slice: *slice,
        translation: rand_point(&mut rng),
        end_gate: None,
        use_entity: None,
    });

    cmd.trigger(trigger::SpawnBuilding {
        name: "store".to_string(),
        slice: *slice,
        translation: rand_point(&mut rng),
        rotation: 0f32,
        alliegance: Alliegance {
            faction: Faction::none(),
            allies: FactionSet::all(),
            enemies: [enemy_faction].into(),
        },
    });

    cmd.trigger(trigger::SpawnBuilding {
        name: "nest".into(),
        translation: rand_point(&mut rng), // TODO
        rotation: 0f32,
        slice: *slice,
        alliegance: Alliegance {
            faction: enemy_faction,
            allies: [enemy_faction].into(),
            enemies: [player_faction].into(),
        },
    });
}

fn manage_slice_transforms(
    mut slices: Query<(&mut Transform, &Slice), Or<(Added<Transform>, Changed<Slice>)>>,
) {
    for (mut transform, slice) in slices.iter_mut() {
        let z = **slice as f32 * -DISTANCE_BETWEEN_SLICES;
        transform.translation.z = z;
    }
}

fn manage_spawners(
    mut cmd: Commands,
    mut spawners: Query<(Entity, &mut Spawner, &Transform, &Slice), Without<Destroyed>>,
    factions: Res<Factions>,
    spawned_from: Query<&SpawnedFrom, Without<Destroyed>>,
    time: Res<Time>,
) {
    let enemy_faction = *factions.get_faction("enemy").unwrap();
    let player_faction = *factions.get_faction("player").unwrap();
    for (entity, mut spawner, transform, slice) in spawners.iter_mut() {
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
                        slice: *slice,
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

/// If anything with a collider comes in contact with the gate, it will change slices
fn manage_gates(
    mut cmd: Commands,
    mut slices: Query<&mut Slice>,
    mut cursor: Local<usize>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gates: Query<(Entity, &Gate, &CollidingEntities)>,
) {
    for (gate_entity, gate, collisions) in gates.iter() {
        if keyboard.just_pressed(KeyCode::KeyF) {
            for collision in collisions.iter() {
                match gate.0 {
                    Some(pair_gate) => {
                        if let Ok(pair_slice) = slices.get(pair_gate).cloned() {
                            if let Ok(mut slice) = slices.get_mut(*collision) {
                                **slice = *pair_slice
                            }
                        }
                    }
                    None => {
                        // We already spawned the first layer, so increase first
                        *cursor += 1;
                        cmd.trigger(trigger::SpawnSlice {
                            from_gate: Some(gate_entity),
                            slice: Slice(*cursor),
                        });
                        if let Ok(mut slice) = slices.get_mut(*collision) {
                            **slice += 1;
                        }
                    }
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
) {
    for (entity, inventory) in changed_chests.iter() {
        if inventory.is_empty() {
            cmd.entity(entity).despawn();
        }
    }
}
