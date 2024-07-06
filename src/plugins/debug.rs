use bevy::prelude::*;

use crate::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_controllers,));
    }
}

fn draw_controllers(mut gizmos: Gizmos, controllers: Query<&Transform, With<Controller>>) {
    controllers.iter().for_each(|transform| {
        let pos = transform.translation;
        let f = transform.forward();
        gizmos.cuboid(*transform, Color::WHITE);
        gizmos.arrow(pos - *f, pos + *f, Color::WHITE);
    });
}
