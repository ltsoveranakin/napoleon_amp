use eframe::egui::Ui;
use std::path::Path;

use napoleon_amp_core::content::song::song_data::SongData;
use napoleon_amp_core::paths::show_file_in_explorer;

pub(crate) mod folder_list;
pub(crate) mod playlist_panel;
pub(crate) mod queue_panel;
pub(crate) mod top_menu_bar;

fn get_song_data_display_str(song_data: &SongData) -> String {
    format!("{} - [{}]", song_data.title, song_data.album)
}

fn open_location_button(ui: &mut Ui, variant_text: &str, path: impl AsRef<Path>) {
    if ui
        .button(format!("Open {} location", variant_text))
        .clicked()
    {
        show_file_in_explorer(path).expect("Error showing file in explorer")
    }
}
