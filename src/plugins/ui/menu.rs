use crate::prelude::*;
use bevy::prelude::*;
use bevy_egui::*;
use bevy_etcetera::Directories;

pub(super) fn draw_main_menu(
    mut contexts: EguiContexts,
    mut state: ResMut<NextState<AppState>>,
    directories: Res<Directories>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        //
        if ui.button("New").clicked() {
            state.set(AppState::NewGame);
        }

        for dir in directories.data_dir().read_dir().unwrap() {
            let dir = dir.unwrap();
            if let Some(extension) = dir.path().extension() {
                if extension.to_os_string().to_str() == Some("ron")
                    && ui
                        .button(format!("Load: {}", dir.file_name().into_string().unwrap()))
                        .clicked()
                {
                    state.set(AppState::LoadGame { path: dir.path() });
                }
            }
        }
    });
}
