use eframe::egui::scroll_area::ScrollSource;
use eframe::egui::ScrollArea;

pub(crate) fn default_scroll_area() -> ScrollArea {
    ScrollArea::vertical().scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
}
