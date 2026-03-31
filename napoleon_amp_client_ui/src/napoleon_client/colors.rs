use eframe::egui::Color32;

pub(crate) const DEFAULT_TEXT_COLOR: Color32 = Color32::PLACEHOLDER;

pub(crate) const SELECTED_TEXT_COLOR: Color32 = Color32::from_rgb(67, 64, 237);
pub(crate) const CURRENT_PLAYING_TEXT_COLOR: Color32 = Color32::from_rgb(222, 64, 2);
pub(crate) const SELECTED_AND_CURRENT_PLAYING_COLOR: Color32 = average(SELECTED_TEXT_COLOR, CURRENT_PLAYING_TEXT_COLOR);

static SEL_TEXT_COLORS: [Color32; 4] = [DEFAULT_TEXT_COLOR, SELECTED_TEXT_COLOR, CURRENT_PLAYING_TEXT_COLOR, SELECTED_AND_CURRENT_PLAYING_COLOR];


const fn average(a: Color32, b: Color32) -> Color32 {
    Color32::from_rgb((a.r().saturating_add(b.r())) / 2, (a.g().saturating_add(b.g())) / 2, (a.b().saturating_add(b.b())) / 2)
}

pub(crate) fn text_color(is_selected: bool, is_currently_playing: bool) -> Color32 {
    SEL_TEXT_COLORS[is_selected as usize + (is_currently_playing as usize * 2)]
}
