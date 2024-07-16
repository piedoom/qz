use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use events::WorldEvent;
use leafwing_input_manager::prelude::*;
use thiserror::Error;

use crate::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        use crate::components;
        app
            // .register_type::<components::Alliegance>()
            .register_type::<components::Chest>()
            .register_type::<components::ChestsInRange>()
            .register_type::<components::Controller>()
            .register_type::<components::Craft>()
            .register_type::<components::Damage>()
            .register_type::<components::Destroyed>()
            .register_type::<components::Drops>()
            .register_type::<components::DropRate>()
            .register_type::<components::Equipment>()
            .register_type::<components::EquipmentType>()
            // .register_type::<components::Faction>()
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
                )
                    .run_if(in_state(AppState::main())),
            );
    }
}

fn setup(mut cmd: Commands, library: Res<Library>, items: Res<Assets<Item>>) {
    // Spawn player
    cmd.spawn((
        Player,
        Name::new("Player"),
        InputManagerBundle::<Action>::default(),
        ChestsInRange {
            chests: default(),
            range: 5f32,
        },
        CraftBundle {
            alliegance: Alliegance {
                faction: Faction::PLAYER,
                allies: Faction::PLAYER,
                enemies: Faction::ENEMY,
            },
            inventory: Inventory::default(),
            equipment: Equipment {
                inventory: Inventory::default()
                    .with_many_single(
                        &["minireactor.energy", "dart.weapon", "autoweld.repair"],
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
    cmd.spawn((
        Structure,
        Health::from(1500),
        Damage::default(),
        Mass(100000f32),
        Collider::sphere(2f32),
        Alliegance {
            faction: Faction::PLAYER,
            allies: Faction::PLAYER,
            enemies: Faction::ENEMY,
        },
        CollisionLayers {
            memberships: LayerMask::from([PhysicsCategory::Structure]),
            filters: LayerMask::from([PhysicsCategory::Weapon, PhysicsCategory::Structure]),
        },
        LockedAxes::ROTATION_LOCKED,
        Transform::default_z(),
    ));

    // spawn nest
    let spawn_nest = |cmd: &mut Commands, position: Vec2, slice: usize| {
        cmd.spawn((
            Structure,
            Spawner {
                maximum: 4,
                delay: Duration::from_secs(3),
                last_spawned: default(),
            },
            Slice(slice),
            Health::from(500),
            Damage::default(),
            RigidBody::Dynamic,
            Mass(100000f32),
            Collider::sphere(1f32),
            Alliegance {
                faction: Faction::ENEMY,
                allies: Faction::ENEMY,
                enemies: Faction::PLAYER,
            },
            CollisionLayers {
                memberships: LayerMask::from([PhysicsCategory::Structure]),
                filters: LayerMask::from([PhysicsCategory::Weapon, PhysicsCategory::Structure]),
            },
            LockedAxes::ROTATION_LOCKED,
            Transform::default_z().with_translation(position.extend(0f32)),
        ));
    };

    // spawn gate
    cmd.spawn((
        Structure,
        Sensor,
        Collider::sphere(2.0),
        CollisionLayers {
            memberships: LayerMask::ALL,
            filters: LayerMask::ALL,
        },
        Slice(0),
        Gate::new(Slice(1)),
        Transform::from_xyz(-10f32, -10f32, 0f32),
    ));

    // spawn gate
    cmd.spawn((
        Structure,
        Sensor,
        Collider::sphere(2.0),
        CollisionLayers {
            memberships: LayerMask::ALL,
            filters: LayerMask::ALL,
        },
        Slice(1),
        Gate::new(Slice(2)),
        Transform::from_xyz(10f32, 5f32, 0f32),
    ));

    // spawn gate
    cmd.spawn((
        Structure,
        Sensor,
        Collider::sphere(2.0),
        CollisionLayers {
            memberships: LayerMask::ALL,
            filters: LayerMask::ALL,
        },
        Slice(2),
        Gate::new(Slice(0)),
        Transform::from_xyz(-10f32, 15f32, 0f32),
    ));

    spawn_nest(&mut cmd, (10f32, -8f32).into(), 1);
    spawn_nest(&mut cmd, (-4f32, 2f32).into(), 1);
    spawn_nest(&mut cmd, (-10f32, 8f32).into(), 2);
    spawn_nest(&mut cmd, (4f32, 2f32).into(), 2);
    spawn_nest(&mut cmd, (6f32, 4f32).into(), 2);
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
    mut spawners: Query<(Entity, &mut Spawner, &Transform, &Slice), Without<Destroyed>>,
    mut events: EventWriter<WorldEvent>,
    spawned_from: Query<&SpawnedFrom, Without<Destroyed>>,
    time: Res<Time>,
) {
    for (entity, mut spawner, transform, slice) in spawners.iter_mut() {
        let new_time = spawner.last_spawned + spawner.delay;
        if time.elapsed() >= new_time {
            if spawned_from.iter().filter(|s| s.0 == entity).count() < spawner.maximum {
                // Spawn thing
                events.send(WorldEvent::SpawnCreature {
                    name: "pest",
                    transform: *transform,
                    slice: slice.0,
                    alliegance: Alliegance {
                        faction: Faction::ENEMY,
                        allies: Faction::ENEMY,
                        enemies: Faction::PLAYER,
                    },
                    from: Some(entity),
                });
                spawner.last_spawned = time.elapsed();
            }
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
    mut events: EventReader<WorldEvent>,
    library: Res<Library>,
    creatures: Res<Assets<Creature>>,
    crafts: Res<Assets<Craft>>,
    items: Res<Assets<Item>>,
) -> Result<(), WorldEventError> {
    for event in events.read() {
        match event {
            WorldEvent::SpawnCreature {
                name,
                transform,
                slice,
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
                            .and_then(|item| items.get(item).cloned())
                            .and_then(|item| Some((item, drop_rate)))
                    })
                    .collect();
                let mut ent = cmd.spawn((
                    CraftBundle {
                        collider: Collider::sphere(craft.size * 0.5),
                        mass: Mass(craft.mass),
                        craft: craft.clone(),
                        transform: *transform,
                        alliegance: *alliegance,
                        inventory: Inventory::with_capacity(craft.capacity)
                            .with_many(inventory.into_iter().collect(), &items, &library)
                            .unwrap(),
                        // TODO: figure out interplay between two capacities
                        equipment: Equipment {
                            inventory: Inventory::with_capacity(craft.capacity)
                                .with_many(equipped.into_iter().collect(), &items, &library)
                                .unwrap(),
                        },
                        slice: Slice(*slice),
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
        }
    }
    Ok(())
}

#[derive(Error, Debug)]
enum WorldEventError {
    #[error("could not find asset with key {0}")]
    AssetNotFound(String),
}
