use eframe::egui::scroll_area::ScrollSource;
use eframe::egui::{ScrollArea, TextWrapMode, Ui};

pub(crate) fn scroll_area_styled(
    ui: &mut Ui,
    scroll_area: ScrollArea,
    add_contents: impl FnOnce(&mut Ui),
) {
    scroll_area
        .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
        .show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);

            add_contents(ui);
        });
}

pub(crate) fn default_scroll_area(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    scroll_area_styled(ui, ScrollArea::vertical(), add_contents);
}
