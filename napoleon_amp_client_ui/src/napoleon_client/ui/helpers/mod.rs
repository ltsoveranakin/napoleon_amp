mod custom_scroll_area;

use eframe::egui::scroll_area::ScrollSource;
use eframe::egui::{Button, IntoAtoms, ScrollArea, TextWrapMode, Ui};
use std::marker::PhantomData;

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

// pub(crate) fn default_scroll_area(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
//     scroll_area_styled(ui, ScrollArea::vertical(), add_contents);
// }

pub(crate) struct ScrollListDisplay<'a, A>
where
    A: IntoAtoms<'a>,
{
    selected: bool,
    display_text: A,
    _phantom: PhantomData<&'a A>,
}

impl<'a, A> ScrollListDisplay<'a, A>
where
    A: IntoAtoms<'a>,
{
    pub(crate) fn new(selected: bool, display_text: A) -> Self {
        Self {
            selected,
            display_text,
            _phantom: PhantomData,
        }
    }
}

pub(crate) fn scroll_area_iter<'list, N, A, I>(
    ui: &mut Ui,
    scroll_area: ScrollArea,
    iterator: I,
    iterator_length: usize,
    get_display: impl Fn(usize, &'list N) -> ScrollListDisplay<'list, A>,
    on_click: impl Fn(usize),
    on_double_click: impl Fn(usize),
) where
    N: 'list,
    A: IntoAtoms<'list> + 'list,
    I: IntoIterator<Item = &'list N>,
{
    scroll_area_styled(ui, scroll_area, |ui| {
        for (i, el) in iterator.into_iter().enumerate() {
            let display = get_display(i, el);

            let button = Button::new(display.display_text)
                .selected(display.selected)
                .frame(true)
                .frame_when_inactive(false);

            let button_response = ui.add(button);

            if button_response.clicked() {
                on_click(i);
            }

            if button_response.double_clicked() {
                on_double_click(i);
            }

            if i != iterator_length - 1 {
                ui.separator();
            }
        }
    });
}
