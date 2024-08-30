use crate::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use events::DockEvent;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionState<Action>>()
            .add_plugins(InputManagerPlugin::<Action>::default())
            .add_plugins(InputManagerPlugin::<AppAction>::default())
            .insert_resource(ActionState::<AppAction>::default())
            .insert_resource(InputMap::<AppAction>::default())
            .add_systems(
                Update,
                (
                    apply_app_input,
                    apply_player_input,
                    //update_player_bindings
                )
                    .run_if(resource_exists::<Library>),
            )
            .add_systems(
                Update,
                (update_app_action_bindings,).run_if(resource_exists_and_changed::<Library>),
            );
    }
}

// fn update_player_bindings(
//     mut cmd: Commands,
//     players: Query<Entity, With<Player>>,
//     new_players: Query<
//         (),
//         (
//             With<Player>,
//             Or<(Without<InputMap<Action>>, Without<ActionState<Action>>)>,
//         ),
//     >,
//     library: Res<Library>,
//     settings: Res<Assets<Settings>>,
// ) {
//     if settings.is_changed() || !new_players.is_empty() {
//         for player in players.iter() {
//             let settings = settings.get(&library.settings).unwrap();
//             cmd.entity(player).insert(InputManagerBundle::with_map(
//                 InputMap::default()
//                     .with_axis(
//                         Action::Turn,
//                         KeyboardVirtualAxis::new(
//                             settings.controls.keyboard.left,
//                             settings.controls.keyboard.right,
//                         ),
//                     )
//                     .with_axis(
//                         Action::Thrust,
//                         KeyboardVirtualAxis::new(
//                             settings.controls.keyboard.brake,
//                             settings.controls.keyboard.thrust,
//                         ),
//                     )
//                     .with(Action::Fire, settings.controls.keyboard.fire)
//                     .with(Action::Take, settings.controls.keyboard.take)
//                     .with(Action::Interact, settings.controls.keyboard.interact),
//             ));
//         }
//     }
// }

fn update_app_action_bindings(
    mut app_actions: ResMut<InputMap<AppAction>>,
    library: Res<Library>,
    settings: Res<Assets<Settings>>,
) {
    InputManagerPlugin::<AppAction>::default();
    // Resource level input
    *app_actions = InputMap::default().with(
        AppAction::Console,
        settings
            .get(&library.settings)
            .unwrap()
            .controls
            .keyboard
            .console,
    );
}

fn apply_app_input(mut draw_inspector: ResMut<DrawInspector>, input: Res<ActionState<AppAction>>) {
    if input.just_pressed(&AppAction::Console) {
        **draw_inspector = !**draw_inspector;
    }
}

/// Apply desired input to the player controller
fn apply_player_input(
    mut cmd: Commands,
    mut players: Query<
        (
            Entity,
            &ActionState<Action>,
            &mut Controller,
            Option<&Children>,
            &Transform,
            &ChestsInRange,
            &DockInRange,
            Option<&Docked>,
        ),
        With<Player>,
    >,
    mut weapons: Query<&mut Weapon>,
    mut dock_events: EventWriter<DockEvent>,
    mut credits: Query<&mut Credits>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    for (
        player_entity,
        actions,
        mut controller,
        maybe_children,
        transform,
        chests_in_range,
        dock_in_range,
        maybe_docked,
    ) in players.iter_mut()
    {
        controller.angular_thrust = actions.clamped_value(&Action::Turn);

        let thrust = actions.clamped_value(&Action::Thrust);
        // There's definitely a prettier way to do this and i will find it out at some point and do it
        match thrust.is_sign_positive() {
            true => {
                // We are thrusting
                controller.brake = 0f32;
                controller.thrust = thrust;
            }
            false => {
                // We are braking
                controller.thrust = 0f32;
                controller.brake = -thrust;
            }
        }

        // Get cursor position
        let cursor_position = match camera.get_single() {
            Ok((camera, camera_transform)) => match window.single().cursor_position() {
                Some(viewport_position) => {
                    if let Some(ray) = camera.viewport_to_world(camera_transform, viewport_position)
                    {
                        let toi = ray
                            .intersect_plane(transform.translation, InfinitePlane3d::new(Vec3::Z));
                        toi.map(|toi| ray.get_point(toi))
                    } else {
                        None
                    }
                }
                None => None,
            },
            Err(_) => None,
        };

        // Get all weapons attached to the player
        if let Some(children) = maybe_children {
            for child in children.iter() {
                if let Ok(mut weapon) = weapons.get_mut(*child) {
                    weapon.wants_to_fire = actions.pressed(&Action::Fire);
                    weapon.target = cursor_position;
                }
            }
        }

        // Take all nearby items
        if actions.just_pressed(&Action::Take) {
            for chest in chests_in_range.chests.iter() {
                if let Ok([mut player_credits, mut chest_credits]) =
                    credits.get_many_mut([player_entity, *chest])
                {
                    // credits chest
                    let amount = chest_credits.get();
                    chest_credits.transfer(&mut player_credits, amount).unwrap();
                } else {
                    // item chest
                    cmd.trigger(triggers::InventoryTransfer {
                        from: *chest,
                        to: player_entity,
                        transfer: triggers::InventoryTransferSettings::All,
                    });
                }
            }
        }

        // Dock/undock at a station

        if actions.just_pressed(&Action::Interact) {
            match maybe_docked {
                Some(_) => {
                    // already docked. Remove
                    dock_events.send(DockEvent::Undock {
                        to_undock: player_entity,
                    });
                }
                None => {
                    // Attempt to dock
                    if let Some(dock) = dock_in_range.dock {
                        dock_events.send(DockEvent::Dock {
                            to_dock: player_entity,
                            dock,
                        });
                    }
                }
            }
        }
    }
}
