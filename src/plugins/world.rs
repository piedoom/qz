use std::time::Duration;

use avian3d::{math::TAU, prelude::*};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_turborand::prelude::*;
use events::WorldEvent;
use leafwing_input_manager::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use thiserror::Error;

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
            .register_type::<components::Energy>()
            .register_type::<components::Equipment>()
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
            .add_event::<WorldEvent>()
            .insert_resource(ClearColor(Color::BLACK))
            .add_systems(OnEnter(AppState::main()), setup)
            .add_systems(
                Update,
                (
                    manage_spawners,
                    manage_world_events.pipe(handle_errors::<WorldEventError>),
                    manage_slice_transforms.after(manage_gates),
                    manage_gates,
                    setup_health,
                    spawn_new_slices,
                )
                    .run_if(in_state(AppState::main())),
            );
    }
}

fn setup(
    mut cmd: Commands,
    mut events: EventWriter<WorldEvent>,
    mut factions: ResMut<Factions>,
    library: Res<Library>,
    items: Res<Assets<Item>>,
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
        InputManagerBundle::<Action>::default(),
        ChestsInRange {
            chests: default(),
            range: 5f32,
        },
        DockInRange {
            dock: None,
            range: 5f32,
        },
        CraftBundle {
            alliegance: player_alliegance.clone(),
            inventory: Inventory::default(),
            equipment: Equipment {
                inventory: Inventory::with_capacity(55)
                    .with_many_from_str(
                        [
                            ("minireactor.energy".to_string(), 1),
                            ("dart.weapon".to_string(), 1),
                            ("autoweld.repair".to_string(), 1),
                        ]
                        .into(),
                        &items,
                        &library,
                    )
                    .unwrap(),
            },
            ..default()
        },
    ));

    // Spawn camera
    cmd.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0f32, -2f32, 48.0).looking_at(Vec3::ZERO, Dir3::Z),
        ..default()
    });

    // spawn base
    let (_, scrap_metal_handle) = item("scrap_metal.item", &items, &library).unwrap();
    cmd.spawn((
        Structure,
        Credits::new(100_000),
        Store {
            items: [(
                scrap_metal_handle,
                SaleOptions {
                    sell: SaleOption::Scaled(1.0),
                    buy: SaleOption::Scaled(0.7),
                },
            )]
            .into(),
        },
        Inventory::with_capacity(1000),
        Health::from(1500),
        Damage::default(),
        Mass(100000f32),
        Collider::sphere(2f32),
        RigidBody::Dynamic,
        player_alliegance.clone(),
        Dockings::default(),
        CollisionLayers {
            memberships: LayerMask::from([PhysicsCategory::Structure]),
            filters: LayerMask::from([PhysicsCategory::Weapon, PhysicsCategory::Structure]),
        },
        LockedAxes::ROTATION_LOCKED,
        Transform::default_z(),
    ));

    // // spawn gates
    // for (translation, (from, to)) in [
    //     (Vec2::new(-10f32, -10f32), (0, 1)),
    //     (Vec2::new(10f32, 5f32), (1, 2)),
    //     (Vec2::new(10f32, 15f32), (2, 0)),
    // ]
    // .into_iter()
    // {
    //     events.send(WorldEvent::SpawnGate {
    //         from: from.into(),
    //         to: to.into(),
    //         translation,
    //         radius: 2.0,
    //     });
    // }

    // for (translation, slice) in [
    //     (Vec2::new(10f32, -8f32), 1),
    //     (Vec2::new(-4f32, 2f32), 1),
    //     (Vec2::new(-10f32, 8f32), 2),
    //     (Vec2::new(4f32, 2f32), 2),
    //     (Vec2::new(6f32, 4f32), 2),
    // ]
    // .into_iter()
    // {
    //     events.send(WorldEvent::SpawnBuilding {
    //         name: "nest".into(),
    //         slice: Slice(slice),
    //         translation,
    //         rotation: 0f32,
    //         alliegance: Alliegance {
    //             faction: enemy_faction,
    //             allies: [enemy_faction].into(),
    //             enemies: [player_faction].into(),
    //         },
    //     });
    // }
}

fn manage_slice_transforms(
    mut slices: Query<
        (&mut Transform, &Slice, Option<&mut Position>),
        Or<(Added<Transform>, Changed<Slice>)>,
    >,
) {
    for (mut transform, slice, maybe_position) in slices.iter_mut() {
        let z = **slice as f32 * -DISTANCE_BETWEEN_SLICES;
        transform.translation.z = z;
        if let Some(mut position) = maybe_position {
            position.z = z;
        }
    }
}

fn manage_spawners(
    mut events: EventWriter<WorldEvent>,
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
                    events.send(WorldEvent::SpawnCreature {
                        name: spawn.clone(),
                        slice: *slice,
                        translation: transform.translation.truncate(),
                        rotation: rng.gen_range(0f32..TAU),
                        alliegance: Alliegance {
                            faction: enemy_faction,
                            allies: [enemy_faction].into(),
                            enemies: [player_faction].into(),
                        },
                        from: Some(entity),
                    });
                    break;
                }
            }

            spawner.last_tick = time.elapsed();
        }
    }
}

/// If anything with a collider comes in contact with the gate, it will change slices
fn manage_gates(gates: Query<(&Gate, &CollidingEntities)>, mut objects: Query<&mut Slice>) {
    for (gate, collisions) in gates.iter() {
        for collision in collisions.iter() {
            if let Ok(mut slice) = objects.get_mut(*collision) {
                **slice = ***gate;
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

fn manage_world_events(
    mut cmd: Commands,
    mut events: ParamSet<(EventReader<WorldEvent>, EventWriter<WorldEvent>)>,
    mut rng: ResMut<GlobalRng>,
    library: Res<Library>,
    creatures: Res<Assets<Creature>>,
    buildings: Res<Assets<Building>>,
    crafts: Res<Assets<Craft>>,
    items: Res<Assets<Item>>,
    factions: Res<Factions>,
) -> Result<(), WorldEventError> {
    let mut new_events: Vec<WorldEvent> = Default::default();

    for event in events.p0().read() {
        match event {
            WorldEvent::SpawnCreature {
                name,
                slice,
                translation,
                rotation,
                alliegance,
                from,
            } => {
                let creature = library
                    .creatures
                    .get(&format!("creatures/{}.creature.ron", name))
                    .ok_or_else(|| WorldEventError::AssetNotFound(name.to_string()))?;
                let Creature {
                    name,
                    craft,
                    drops,
                    inventory,
                    equipped,
                    range,
                } = creatures
                    .get(creature)
                    .cloned()
                    .ok_or_else(|| WorldEventError::AssetNotFound(name.to_string()))?;
                let craft = library
                    .crafts
                    .get(&format!("crafts/{}.craft.ron", craft))
                    .and_then(|craft| crafts.get(craft))
                    .ok_or_else(|| WorldEventError::AssetNotFound(name.to_string()))?;
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
                        // TODO: figure out interplay between two capacities
                        equipment: Equipment {
                            inventory: Inventory::with_capacity(craft.capacity)
                                .with_many_from_str(
                                    equipped.into_iter().collect(),
                                    &items,
                                    &library,
                                )
                                .unwrap(),
                        },
                        slice: *slice,
                        ..default()
                    },
                    Npc,
                    Drops(drops),
                    InRange::new(range),
                ));
                if let Some(from) = from {
                    ent.insert((SpawnedFrom(*from),));
                }
            }
            WorldEvent::SpawnSlice(slice) => {
                const SEPARATION_SCALAR: f32 = 24.0;
                let gate_pos = {
                    Vec2::new(rng.f32() - 0.5, rng.f32() - 0.5)
                        * rng.f32()
                        * ((**slice as f32 * 0.1) + 1.0)
                        * SEPARATION_SCALAR
                };

                new_events.push(WorldEvent::SpawnGate {
                    from: *slice,
                    to: (**slice + 1).into(),
                    translation: gate_pos,
                    radius: 2.0,
                });

                let player_faction = *factions.get_faction("player").unwrap();
                let enemy_faction = *factions.get_faction("enemy").unwrap();

                let nest_pos = {
                    Vec2::new(rng.f32() - 0.5, rng.f32() - 0.5)
                        * rng.f32()
                        * ((**slice as f32 * 0.1) + 1.0)
                        * SEPARATION_SCALAR
                };

                new_events.push(WorldEvent::SpawnBuilding {
                    name: "nest".into(),
                    translation: nest_pos, // TODO
                    rotation: 0f32,
                    slice: *slice,
                    alliegance: Alliegance {
                        faction: enemy_faction,
                        allies: [enemy_faction].into(),
                        enemies: [player_faction].into(),
                    },
                })
            }
            WorldEvent::SpawnBuilding {
                name,
                translation,
                rotation,
                slice,
                alliegance,
            } => {
                let building = library
                    .building(name)
                    .and_then(|building| buildings.get(building.id()))
                    .ok_or_else(|| WorldEventError::AssetNotFound(name.to_string()))?
                    .clone();

                let mut entity = cmd.spawn((
                    Name::new(name.clone()),
                    Structure,
                    *slice,
                    Health::from(building.health),
                    Damage::default(),
                    RigidBody::Dynamic,
                    Mass(100000f32),
                    Collider::sphere(1f32),
                    alliegance.clone(),
                    CollisionLayers {
                        memberships: LayerMask::from([PhysicsCategory::Structure]),
                        filters: LayerMask::from([
                            PhysicsCategory::Weapon,
                            PhysicsCategory::Structure,
                        ]),
                    },
                    LockedAxes::ROTATION_LOCKED,
                    Transform::z_from_parts(translation, rotation, slice),
                ));

                if let Some(spawner) = building.spawner {
                    entity.insert((spawner,));
                }
            }
            WorldEvent::SpawnGate {
                from,
                to,
                translation,
                radius: size,
            } => {
                cmd.spawn((
                    Structure,
                    Sensor,
                    Collider::sphere(*size),
                    CollisionLayers {
                        memberships: LayerMask::ALL,
                        filters: LayerMask::ALL,
                    },
                    *from,
                    Gate::new(*to),
                    Transform::z_from_parts(translation, &0f32, from),
                ));
            }
        }
    }

    // Send any events queued while running
    let mut writer = events.p1();
    for new_event in new_events.into_iter() {
        writer.send(new_event);
    }
    // Success!
    Ok(())
}

#[derive(Error, Debug)]
enum WorldEventError {
    #[error("could not find asset with key {0}")]
    AssetNotFound(String),
}

/// Keeps track of the furthest player and spawns a slice in advance.
fn spawn_new_slices(
    mut events: EventWriter<WorldEvent>,
    mut cursor: Local<Slice>,
    players: Query<&Slice, With<Player>>,
) {
    const SLICES_IN_ADVANCE: usize = 3;
    let furthest_player = players
        .iter()
        .fold(0, |acc, p| if p.0 > acc { p.0 } else { acc });

    for slice in **cursor..furthest_player + SLICES_IN_ADVANCE {
        events.send(WorldEvent::SpawnSlice(slice.into()));
    }

    **cursor = furthest_player + SLICES_IN_ADVANCE;
}
