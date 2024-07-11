use crate::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionState<Action>>()
            .insert_resource(
                InputMap::<Action>::default(), // Insert the control map here
            )
            .add_plugins(InputManagerPlugin::<Action>::default())
            .add_systems(Update, apply_player_input);
    }
}

/// Apply desired input to the player controller
fn apply_player_input(
    mut players: Query<
        (&ActionState<Action>, &mut Controller, &Children, &Transform),
        With<Player>,
    >,
    mut weapons: Query<&mut Weapon>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    for (actions, mut controller, children, transform) in players.iter_mut() {
        controller.angular_thrust = actions.value(&Action::Turn);
        controller.thrust = actions.value(&Action::Thrust);
        controller.brake = actions.value(&Action::Brake);

        // Get cursor position
        let cursor_position = match camera.get_single() {
            Ok((camera, camera_transform)) => match window.single().cursor_position() {
                Some(viewport_position) => {
                    let ray = camera
                        .viewport_to_world(camera_transform, viewport_position)
                        .unwrap();
                    let toi =
                        ray.intersect_plane(transform.translation, InfinitePlane3d::new(Vec3::Z));
                    toi.map(|toi| ray.get_point(toi))
                }
                None => None,
            },
            Err(_) => None,
        };

        // Get all weapons attached to the player
        for child in children.iter() {
            if let Ok(mut weapon) = weapons.get_mut(*child) {
                weapon.wants_to_fire = actions.value(&Action::Fire) != 0f32;
                weapon.target = cursor_position;
            }
        }
    }
}
