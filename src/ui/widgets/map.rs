use crate::prelude::{Gate, Universe, UniverseGraph, UniversePosition};
use avian3d::prelude::Collider;
use bevy::{
    prelude::{GlobalTransform, Query, Res},
    utils::HashMap,
};
use bevy_egui::egui::*;
use petgraph::{graph::NodeIndex, visit::EdgeRef};

/// A map of the currently loaded zone
pub struct ZoneMap<'a> {
    /// Size of the map
    pub size: Vec2,
    /// Overall zoom of the map where 1 unit = 1 pixel when scale is 1
    pub scale: f32,
    /// Data to ingest and display
    pub collider_query: &'a Query<'a, 'a, (&'static GlobalTransform, &'static Collider)>,
    /// Gates
    pub gate_query: &'a Query<'a, 'a, (&'static GlobalTransform, &'static Gate)>,
    /// Center position
    pub world_center: Vec2,
    /// Universe
    pub universe_graph: &'a UniverseGraph,
}

impl<'a> Widget for ZoneMap<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            scale,
            collider_query: query,
            size,
            world_center,
            gate_query,
            universe_graph,
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

        for (transform, gate) in gate_query.iter() {
            let pos = world_to_px(transform);
            painter.circle(pos, 4f32, Color32::BLUE, Stroke::NONE);

            let gate_zone = universe_graph.node_weight(gate.destination()).unwrap();
            painter.text(
                pos,
                Align2::CENTER_TOP,
                gate_zone.name.replace(" ", "\n"),
                FontId::monospace(8f32),
                Color32::WHITE,
            );
        }

        response
    }
}

/// Universe minimap
#[derive(Default)]
pub struct UniverseMap<'a> {
    /// Requested size of the minimap
    pub size: Vec2,
    /// Underlying universe graph representation
    pub graph: Option<&'a UniverseGraph>,
    /// Current position, if any, in the graph
    pub current_position: Option<Res<'a, UniversePosition>>,
}

impl<'a> Widget for UniverseMap<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        #[allow(unused_assignments)]
        let mut node_index_pos: HashMap<NodeIndex, Pos2> = Default::default();

        let Self {
            size,
            graph,
            current_position,
        } = self;
        let (rect, response) = ui.allocate_at_least(size, Sense::hover());
        let painter = ui.painter().with_clip_rect(rect);

        if let Some(graph) = graph {
            node_index_pos = {
                let nodes = graph
                    .node_weights()
                    .zip(graph.node_indices())
                    .collect::<Vec<_>>();

                let get_pos = |layer_index: usize, total_in_layer: usize, depth: usize| -> Pos2 {
                    let segment_width = rect.width() / total_in_layer as f32;
                    let current_segment_center_x =
                        (segment_width * (layer_index + 1) as f32) - (segment_width / 2f32);
                    rect.left_top()
                        + (current_segment_center_x, (depth as f32 + 1f32) * 48f32).into()
                };

                nodes
                    .chunk_by(|a, b| a.0.depth == b.0.depth)
                    .flat_map(|layer| {
                        layer.iter().enumerate().map(|(i, (zone, node_index))| {
                            (*node_index, get_pos(i, layer.len(), zone.depth))
                        })
                    })
                    .collect::<HashMap<_, _>>()
            };

            for (node_index, zone) in graph.node_indices().zip(graph.node_weights()) {
                let is_current_node = current_position
                    .as_ref()
                    .map(|cur| cur.0 == node_index)
                    .unwrap_or_default();

                let color = if is_current_node {
                    Color32::RED
                } else {
                    Color32::from_white_alpha(50)
                };

                // paint edges
                let edges = graph
                    .edges(node_index)
                    .map(|edge| graph.edge_endpoints(edge.id()).unwrap());

                for (a, b) in edges {
                    let pos_a = node_index_pos.get(&a).unwrap();
                    let pos_b = node_index_pos.get(&b).unwrap();
                    painter.line_segment([*pos_a, *pos_b], Stroke::new(1f32, Color32::WHITE));
                }

                painter.circle(
                    *node_index_pos.get(&node_index).unwrap(),
                    if is_current_node { 8f32 } else { 4f32 },
                    color,
                    Stroke::NONE,
                );

                // paint in the center of the segment width
                if zone.scene.is_some() || is_current_node {
                    painter.text(
                        *node_index_pos.get(&node_index).unwrap(),
                        Align2::CENTER_CENTER,
                        zone.name.replace(" ", "\n"),
                        FontId::monospace(if is_current_node { 12f32 } else { 8f32 }),
                        if is_current_node {
                            Color32::WHITE
                        } else {
                            Color32::from_white_alpha(50)
                        },
                    );
                }
            }
        }

        response
    }
}
