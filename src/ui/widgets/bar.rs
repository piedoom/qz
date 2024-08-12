use std::ops::RangeInclusive;

use bevy_egui::egui::*;

use crate::prelude::*;

/// Health bar, shields, etc.
pub struct Bar {
    /// Size of the bar
    pub size: Vec2,
    /// Range to consider when drawing the bar value
    pub range: RangeInclusive<f32>,
    /// Current value of the bar. `range` should contain this.
    pub value: f32,
    /// Border radius
    pub radius: f32,
    /// Bar fill color
    pub fill: Color32,
    /// Bar stroke
    pub stroke: Stroke,
    /// If specified, draw at this position
    pub position: Option<Pos2>,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            size: (100f32, 20f32).into(),
            range: Default::default()..=Default::default(),
            value: Default::default(),
            radius: 3f32,
            fill: Color32::GREEN,
            stroke: Stroke::new(2f32, Color32::WHITE),
            position: None,
        }
    }
}

impl Widget for Bar {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            size,
            range,
            value,
            radius,
            fill,
            stroke,
            position,
        } = self;
        let (rect, response) = match position {
            Some(position) => {
                let rect = Rect::from_center_size(position, size);
                (rect, ui.allocate_rect(rect, Sense::hover()))
            }
            None => ui.allocate_at_least(size, Sense::hover()),
        };

        let painter = ui.painter().with_clip_rect(rect);

        let mut inner_rect = rect;
        inner_rect.set_width(value.normalize(range) * rect.width());

        painter.rect(inner_rect, radius, fill, Stroke::NONE);

        painter.rect(rect, radius, Color32::TRANSPARENT, stroke);

        response
    }
}
