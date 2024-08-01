use crate::prelude::*;
use avian3d::prelude::{
    AngularVelocity, ExternalAngularImpulse, ExternalImpulse, LinearDamping, LinearVelocity,
};
use bevy::prelude::*;

pub struct ControllersPlugin;

impl Plugin for ControllersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                apply_controller_movement,
                apply_craft_physics.after(apply_controller_movement),
            ),
        );
    }
}

/// Apply input to the controllers
fn apply_controller_movement(
    mut characters: Query<
        (
            &Transform,
            &Craft,
            &mut Controller,
            &mut ExternalImpulse,
            &mut AngularVelocity,
            &mut LinearDamping,
        ),
        Without<Destroyed>,
    >,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    characters.iter_mut().for_each(
        |(transform, craft, controller, mut velocity, mut angular, mut damping)| {
            **velocity += controller.thrust * transform.forward() * dt * craft.acceleration;
            **velocity = velocity.clamp_length_max(craft.speed);
            **angular = controller.angular_thrust * -Vec3::Z * dt * craft.rotation;
            **damping = controller.brake * dt * craft.brake;
        },
    );
}

/// Apply limitations of a particular craft
fn apply_craft_physics(
    mut crafts: Query<
        (&mut LinearVelocity, &mut ExternalAngularImpulse, &Craft),
        Without<Destroyed>,
    >,
    time: Res<Time>,
) {
    crafts
        .iter_mut()
        .for_each(|(mut velocity, mut angular_velocity, craft)| {
            // Clamp to max speed
            **velocity = velocity.clamp_length_max(craft.speed);
            // Clamp to max rotation
            **angular_velocity =
                angular_velocity.clamp_length_max(craft.rotation * time.delta_seconds());
        });
}
