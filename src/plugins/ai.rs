use crate::prelude::*;
use avian3d::{
    parry::na::ComplexField,
    prelude::{AngularVelocity, ExternalAngularImpulse, ExternalImpulse, LinearVelocity},
};
use bevy::prelude::*;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (fly_towards_enemy,));
    }
}

fn fly_towards_enemy(
    mut npcs: Query<(&mut Controller, &Transform, &Alliegance), With<Npc>>,
    all_crafts: Query<(&Transform, &Alliegance)>,
) {
    for (mut controller, transform, alliegance) in npcs.iter_mut() {
        // TODO: Don't retarget every frame
        let mut enemies = all_crafts
            .iter()
            .filter(|(_, a)| alliegance.enemies.contains(a.faction));
        if let Some((enemy_transform, _)) = enemies.next() {
            // get the ship forward vector in 2D
            let forward = (transform.rotation * Vec3::Z).xy();

            // get the vector from the ship to the enemy ship in 2D and normalize it.
            let to_enemy =
                (transform.translation.xy() - enemy_transform.translation.xy()).normalize();

            // get the dot product between the enemy forward vector and the direction to the player.
            let forward_dot_enemy = forward.dot(to_enemy);

            // if the dot product is approximately 1.0 then already facing and
            // we can early out.
            if (forward_dot_enemy - 1.0).abs() < f32::EPSILON {
                continue;
            }

            // get the right vector of the ship in 2D (already unit length)
            let right = (transform.rotation * Vec3::X).xy();
            let right_dot_enemy = right.dot(to_enemy);
            let rotation_sign = -f32::copysign(1.0, right_dot_enemy);
            if rotation_sign > 0f32 {
                controller.angular_thrust = 1f32
            } else {
                controller.angular_thrust = -1f32
            }

            if right_dot_enemy.abs() < 0.5 {
                controller.thrust = 1f32
            } else {
                controller.thrust = 0f32
            }
        }
    }
}
