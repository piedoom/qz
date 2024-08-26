use crate::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;

pub(super) fn on_spawn_gate(
    trigger: Trigger<trigger::SpawnGate>,
    mut cmd: Commands,
    library: Res<Library>,
) {
    const GATE_RADIUS: f32 = 2.0f32;
    let trigger::SpawnGate {
        translation,
        destination,
    } = trigger.event();

    cmd.spawn((
        Persistent,
        Structure,
        Sensor,
        Collider::sphere(GATE_RADIUS),
        CollisionLayers {
            memberships: LayerMask::ALL,
            filters: LayerMask::ALL,
        },
        Gate::new(*destination),
        GlobalTransform::IDENTITY,
        Transform::z_from_parts(translation, &0f32),
        Model::new(library.model("structures/gate").unwrap()).with_offset(Vec3::Y * -2f32),
    ));
}
