use eframe::egui::scroll_area::ScrollSource;
use eframe::egui::{ScrollArea, TextWrapMode, Ui};

pub(crate) struct ScrollAreaList {
    scroll_area: ScrollArea,
}

impl ScrollAreaList {
    pub(crate) fn new() -> Self {
        Self {
            scroll_area: ScrollArea::vertical()
                .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR),
        }
    }

    pub(crate) fn show(self, ui: &mut Ui) {
        self.scroll_area.show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        });
    }
}
