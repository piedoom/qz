use avian3d::prelude::Collider;
use bevy::prelude::{GlobalTransform, Query};
use bevy_egui::egui::*;

/// A map of the currently loaded zone
pub struct ZoneMap<'a> {
    /// Size of the map
    pub size: Vec2,
    /// Overall zoom of the map where 1 unit = 1 pixel when scale is 1
    pub scale: f32,
    /// Data to ingest and display
    pub collider_query: &'a Query<'a, 'a, (&'static GlobalTransform, &'static Collider)>,
    /// Center position
    pub world_center: Vec2,
}

impl<'a> Widget for ZoneMap<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            scale,
            collider_query: query,
            size,
            world_center,
        } = self;

        let world_center = world_center * Vec2::new(1f32, -1f32);
        let (rect, response) = ui.allocate_at_least(size, Sense::hover());
        let painter = ui.painter().with_clip_rect(rect);

        let world_to_px = |transform: &GlobalTransform| -> Pos2 {
            let translation = transform.compute_transform().translation.truncate();
            // Relative world translation of this object compared to the world center
            let relative_translation = world_center - (translation.x, -translation.y).into();
            // Draw on the screen
            let px_relative = relative_translation * scale;
            rect.center() - px_relative
        };

        // paint in the middle
        let bb = Rect::from_center_size(rect.center(), Vec2::splat(16f32));
        let stroke = Stroke::new(1f32, Color32::WHITE);
        painter.line_segment([bb.center_top(), bb.center_bottom()], stroke);
        painter.line_segment([bb.left_center(), bb.right_center()], stroke);

        for (transform, _) in query.iter() {
            painter.circle(world_to_px(transform), 2f32, Color32::GREEN, Stroke::NONE);
        }

        response
    }
}
