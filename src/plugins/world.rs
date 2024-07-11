use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

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
            .register_type::<components::Drop>()
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
            .insert_resource(ClearColor(Color::BLACK))
            .add_systems(OnEnter(AppState::main()), setup)
            .add_systems(
                Update,
                (
                    manage_spawners,
                    manage_slice_transforms.after(manage_gates),
                    manage_gates,
                )
                    .run_if(in_state(AppState::main())),
            );
    }
}

fn setup(
    mut cmd: Commands,
    settings: Res<Settings>,
    library: Res<Library>,
    items: Res<Assets<Item>>,
) {
    let item = |name: &str| -> Option<&Item> {
        items.get(library.items.get(&format!("items/{}.ron", name)).unwrap())
    };
    let input_map = InputMap::default()
        .with(
            Action::Turn,
            KeyboardVirtualAxis::new(
                settings.controls.keyboard.left,
                settings.controls.keyboard.right,
            ),
        )
        .with(Action::Thrust, settings.controls.keyboard.thrust)
        .with(Action::Brake, settings.controls.keyboard.brake)
        .with(Action::Fire, settings.controls.keyboard.fire);

    // Spawn player
    cmd.spawn((
        Player,
        Name::new("Player"),
        InputManagerBundle::with_map(input_map),
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
                    .with_many(
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

    spawn_nest(&mut cmd, (13f32, 8f32).into(), 0);
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
    mut cmd: Commands,
    mut spawners: Query<(Entity, &mut Spawner, &Transform, &Slice), Without<Destroyed>>,
    spawned_from: Query<&SpawnedFrom, Without<Destroyed>>,
    library: Res<Library>,
    items: Res<Assets<Item>>,
    time: Res<Time>,
) {
    let metals: Item = Item {
        name: String::from("metals"),
        mass: 1.,
        size: 1,
        equipment: None,
    };

    for (entity, mut spawner, transform, slice) in spawners.iter_mut() {
        let new_time = spawner.last_spawned + spawner.delay;
        if time.elapsed() >= new_time {
            if spawned_from.iter().filter(|s| s.0 == entity).count() < spawner.maximum {
                // Spawn thing
                cmd.spawn((
                    SpawnedFrom(entity),
                    InRange::new(16.0),
                    CraftBundle {
                        craft: Craft {
                            speed: 6f32,
                            rotation: 80f32,
                            brake: 10f32,
                            acceleration: 50f32,
                        },
                        alliegance: Alliegance {
                            faction: Faction::ENEMY,
                            allies: Faction::ENEMY,
                            enemies: Faction::PLAYER,
                        },
                        slice: slice.clone(),
                        transform: *transform,
                        equipment: Equipment {
                            inventory: Inventory::default()
                                .with_many(
                                    &["minireactor.energy", "dart.weapon", "autoweld.repair"],
                                    &items,
                                    &library,
                                )
                                .unwrap(),
                        },
                        ..default()
                    },
                    Npc,
                    Drop {
                        items: [(
                            metals.clone(),
                            DropRate {
                                amount: 10..=20,
                                d: 3,
                            },
                        )]
                        .into(),
                    },
                ));
            }
            spawner.last_spawned = time.elapsed();
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
