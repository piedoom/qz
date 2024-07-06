use crate::prelude::*;
use avian3d::prelude::{
    AngularVelocity, ExternalAngularImpulse, ExternalForce, ExternalImpulse, ExternalTorque,
};
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
fn apply_player_input(mut players: Query<(&ActionState<Action>, &mut Controller), With<Player>>) {
    players.iter_mut().for_each(|(actions, mut controller)| {
        controller.angular_thrust = actions.value(&Action::Turn);
        controller.thrust = actions.value(&Action::Thrust);
        controller.brake = actions.value(&Action::Brake);
    });
}
