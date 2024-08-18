use crate::prelude::*;
use bevy::prelude::*;
use bevy_egui::*;
use egui::Align2;

pub(super) fn draw_hud(
    mut contexts: EguiContexts,
    healths: Query<(&Transform, &Health, &Damage), Without<Destroyed>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    if let Ok((camera, global_transform)) = camera.get_single() {
        egui::Area::new("hud".into())
            .interactable(false)
            .anchor(Align2::LEFT_TOP, egui::Vec2::ZERO)
            .default_size(contexts.ctx_mut().screen_rect().size())
            .show(contexts.ctx_mut(), |ui| {
                for (transform, health, damage) in healths.iter() {
                    if let Some(viewport_position) =
                        camera.world_to_viewport(global_transform, transform.translation)
                    {
                        ui.add(widgets::Bar {
                            size: (32f32, 8f32).into(),
                            range: 0f32..=health.get() as f32,
                            value: health.get() as f32 - **damage,
                            position: Some(egui::Pos2::new(
                                viewport_position.x,
                                viewport_position.y + 32f32,
                            )),
                            ..Default::default()
                        });
                    }
                }
            });
    }
}
