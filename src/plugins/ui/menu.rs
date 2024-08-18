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
            state.set(AppState::New);
        }

        for dir in directories.data_dir().read_dir().unwrap() {
            let dir = dir.unwrap();
            if ui
                .button(format!("Load: {}", dir.file_name().into_string().unwrap()))
                .clicked()
            {
                state.set(AppState::LoadGame(
                    dir.path()
                        .join(dir.file_name())
                        .with_extension("save.ron")
                        .into(),
                ));
            }
        }
    });
}
