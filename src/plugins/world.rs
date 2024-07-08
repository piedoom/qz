use std::time::Duration;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::main()), setup);
    }
}

fn setup(mut cmd: Commands, settings: Res<Settings>) {
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
        CraftBundle {
            alliegance: Alliegance {
                faction: Faction::PLAYER,
                allies: Faction::PLAYER,
                enemies: Faction::ENEMY,
            },
            inventory: Inventory::default()
                .with(
                    Item {
                        name: "metals",
                        mass: 1.,
                        size: 1,
                        equipment: None,
                    },
                    10,
                )
                .unwrap(),
            equipment: Equipment {
                inventory: Inventory::default()
                    .with(
                        Item {
                            name: "gun",
                            mass: 1f32,
                            size: 1,
                            equipment: Some(EquipmentType::Weapon(Weapon {
                                weapon_type: WeaponType::Projectile {
                                    speed: 10.0,
                                    recoil: Duration::from_millis(200),
                                    damage: 20,
                                    radius: 0.1,
                                    spread: 30f32.to_radians(),
                                    shots: 3,
                                    lifetime: Duration::from_secs(4),
                                },
                                wants_to_fire: default(),
                                last_fired: default(),
                            })),
                        },
                        1,
                    )
                    .unwrap()
                    .with(
                        Item {
                            name: "gun2",
                            mass: 1f32,
                            size: 1,
                            equipment: Some(EquipmentType::Weapon(Weapon {
                                weapon_type: WeaponType::Projectile {
                                    speed: 18.0,
                                    recoil: Duration::from_millis(20),
                                    damage: 1,
                                    radius: 0.05,
                                    spread: 1f32.to_radians(),
                                    shots: 1,
                                    lifetime: Duration::from_secs(4),
                                },
                                wants_to_fire: default(),
                                last_fired: default(),
                            })),
                        },
                        1,
                    )
                    .unwrap()
                    .with(
                        Item {
                            name: "repair",
                            mass: 1f32,
                            size: 1,
                            equipment: Some(EquipmentType::RepairBot(RepairBot { rate: 5f32 })),
                        },
                        1,
                    )
                    .unwrap(),
            },
            ..default()
        },
    ));

    // Spawn enemies
    for i in 0..10 {
        cmd.spawn((
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
                transform: Transform::default_z().with_translation(Vec3::new(
                    15.0 + (i as f32 * 2f32),
                    15.0,
                    0.0,
                )),
                equipment: Equipment {
                    inventory: Inventory::default()
                        .with(
                            Item {
                                name: "gun",
                                mass: 1f32,
                                size: 1,
                                equipment: Some(EquipmentType::Weapon(Weapon {
                                    weapon_type: WeaponType::Projectile {
                                        speed: 8.0,
                                        recoil: Duration::from_millis(400),
                                        damage: 5,
                                        spread: 20f32.to_radians(),
                                        shots: 1,
                                        radius: 0.1,
                                        lifetime: Duration::from_secs(2),
                                    },
                                    wants_to_fire: default(),
                                    last_fired: default(),
                                })),
                            },
                            1,
                        )
                        .unwrap(),
                },
                ..default()
            },
            Npc,
        ));
    }

    // Spawn camera
    cmd.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0f32, -2f32, 48.0).looking_at(Vec3::ZERO, Dir3::Z),
        ..default()
    });
}
