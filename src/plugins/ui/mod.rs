mod game;
mod hud;
mod menu;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_egui::*;
use egui::Align2;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use {game::*, hud::*, menu::*};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (draw_ui, draw_toasts, draw_minimaps, draw_hud).run_if(in_state(AppState::main())),
                (draw_main_menu,).run_if(in_state(AppState::menu())),
                (inspector_ui,).run_if(resource_equals::<DrawInspector>(DrawInspector(true))),
            ),
        );
    }
}

fn inspector_ui(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<bevy::window::PrimaryWindow>>()
        .single(world)
        .clone();

    // the usual `ResourceInspector` code
    egui::SidePanel::new(egui::panel::Side::Right, "Resource Inspector").show(
        egui_context.get_mut(),
        |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);

                ui.separator();
                ui.label("Press space to toggle");
            });
        },
    );
}

fn draw_toasts(mut contexts: EguiContexts, mut errors: EventReader<GameError>) {
    let mut toasts = Toasts::new()
        .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0)) // 10 units from the bottom right corner
        .direction(egui::Direction::BottomUp);

    for error in errors.read() {
        toasts.add(Toast {
            text: format!("{error}").into(),
            kind: ToastKind::Error,
            options: ToastOptions::default()
                .duration_in_seconds(5.0)
                .show_progress(true),
            ..Default::default()
        });
    }

    egui::Area::new("toasts".into())
        .interactable(false)
        .anchor(Align2::RIGHT_BOTTOM, (16f32, 16f32))
        .show(contexts.ctx_mut(), |ui| {
            toasts.show(ui.ctx());
        });
}
