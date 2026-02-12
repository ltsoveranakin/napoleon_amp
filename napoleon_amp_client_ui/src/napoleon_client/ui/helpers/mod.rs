mod custom_scroll_area;

use eframe::egui::scroll_area::ScrollSource;
use eframe::egui::{Button, IntoAtoms, ScrollArea, TextWrapMode, Ui};
use napoleon_amp_core::content::NamedPathLike;
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

pub(crate) fn scroll_area_list<'list, N, A>(
    ui: &mut Ui,
    scroll_area: ScrollArea,
    list: &'list [N],
    get_display: impl Fn(usize, &'list N) -> ScrollListDisplay<'list, A>,
    on_click: impl Fn(usize),
    on_double_click: impl Fn(usize),
) where
    A: IntoAtoms<'list> + 'list,
{
    scroll_area_styled(ui, scroll_area, |ui| {
        for (i, el) in list.iter().enumerate() {
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

            if i != list.len() - 1 {
                ui.separator();
            }
        }
    });
}

pub(crate) fn scroll_area_named_list<N>(
    ui: &mut Ui,
    scroll_area: ScrollArea,
    list: &[N],
    is_selected: impl Fn(usize, &N) -> bool,
    on_click: impl Fn(usize),
    on_double_click: impl Fn(usize),
) where
    N: NamedPathLike,
{
    scroll_area_list(
        ui,
        scroll_area,
        list,
        |i, named| ScrollListDisplay::new(is_selected(i, named), named.name()),
        on_click,
        on_double_click,
    );
}

// pub(crate) fn scroll_area_list<N>(
//     ui: &mut Ui,
//     get_display: impl Fn(&N) -> String,
//     on_click: impl FnOnce(usize),
// ) {
//     default_scroll_area(ui, |ui| {
//         for (i, el) in named_list.iter().enumerate() {
//             let text = get_display(el);
//
//
//
//             if i != named_list.len() - 1 {
//                 ui.separator();
//             }
//         }
//     });
// }
