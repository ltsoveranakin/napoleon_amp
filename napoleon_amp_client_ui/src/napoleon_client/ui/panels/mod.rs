use eframe::egui::Ui;
use napoleon_amp_core::content::song::song_data::SongDataStd;
use napoleon_amp_core::paths::show_file_in_explorer;
use std::path::Path;

pub(crate) mod folder_list;
pub(crate) mod playlist_panel;
pub(crate) mod queue_panel;
pub(crate) mod top_menu_bar;

fn get_song_data_display_str(song_data: &SongDataStd) -> String {
    format!(
        "{} - [{}]",
        song_data.title,
        song_data.meta.inner.album.as_ref().unwrap()
    )
}

fn open_location_button(ui: &mut Ui, variant_text: &str, path: impl AsRef<Path>) {
    if ui
        .button(format!("Open {} location", variant_text))
        .clicked()
    {
        show_file_in_explorer(path).expect("Error showing file in explorer");
    }
}

pub(super) enum CloseResult {
    KeepOpen,
    CloseWithoutSaving,
    SaveAndClose,
}

impl CloseResult {
    pub(super) fn should_close(&self) -> bool {
        match self {
            CloseResult::KeepOpen => false,

            CloseResult::CloseWithoutSaving => true,

            CloseResult::SaveAndClose => true,
        }
    }

    pub(super) fn should_save(&self) -> bool {
        match self {
            CloseResult::SaveAndClose => true,

            _ => false,
        }
    }
}
