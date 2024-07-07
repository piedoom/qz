use crate::prelude::*;
use bevy::prelude::*;
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
    mut players: Query<(&ActionState<Action>, &mut Controller, &Children), With<Player>>,
    mut weapons: Query<&mut Weapon>,
) {
    for (actions, mut controller, children) in players.iter_mut() {
        controller.angular_thrust = actions.value(&Action::Turn);
        controller.thrust = actions.value(&Action::Thrust);
        controller.brake = actions.value(&Action::Brake);
        // Get all weapons attached to the player
        for child in children.iter() {
            if let Ok(mut weapon) = weapons.get_mut(*child) {
                weapon.wants_to_fire = actions.value(&Action::Fire) != 0f32;
            }
        }
    }
}
