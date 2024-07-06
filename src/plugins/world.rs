use avian3d::prelude::*;
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
        .with(Action::Brake, settings.controls.keyboard.brake);

    // Spawn player
    cmd.spawn((
        Player,
        InputManagerBundle::with_map(input_map),
        CraftBundle {
            alliegance: Alliegance {
                faction: Faction::PLAYER,
                allies: Faction::PLAYER,
                enemies: Faction::ENEMY,
            },
            ..default()
        },
    ))
    .with_children(|cmd| {
        cmd.spawn(Equipment::default());
    });

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
                ..default()
            },
            Npc,
        ))
        .with_children(|cmd| {
            cmd.spawn(Equipment::default());
        });
    }

    // Spawn camera
    cmd.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0f32, -2f32, 48.0).looking_at(Vec3::ZERO, Dir3::Z),
        ..default()
    });
}
