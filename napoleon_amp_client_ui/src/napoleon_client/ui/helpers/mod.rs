pub(crate) mod custom_modal;
pub(crate) mod select_button;

use crate::napoleon_client::duration_to_str;
use crate::napoleon_client::ui::panels::CloseResult;
use eframe::egui::scroll_area::{ScrollAreaOutput, ScrollSource};
use eframe::egui::{Button, IntoAtoms, ScrollArea, TextWrapMode, Ui};
use std::marker::PhantomData;
use std::str::FromStr;
use std::time::Duration;

pub(crate) fn scroll_area_styled<O>(
    ui: &mut Ui,
    scroll_area: ScrollArea,
    add_contents: impl FnOnce(&mut Ui) -> O,
) -> ScrollAreaOutput<O> {
    scroll_area
        .scroll_source(ScrollSource::MOUSE_WHEEL | ScrollSource::SCROLL_BAR)
        .show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);

            add_contents(ui)
        })
}

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

pub(super) fn close_ui(ui: &mut Ui) -> CloseResult {
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            return CloseResult::SaveAndClose;
        }

        if ui.button("Cancel").clicked() {
            return CloseResult::CloseWithoutSaving;
        }

        CloseResult::KeepOpen
    })
    .inner
}

pub(super) fn duration_input(ui: &mut Ui, duration: &mut Duration) {
    let mut duration_str = duration_to_str(*duration);

    if ui.text_edit_singleline(&mut duration_str).changed() {
        let duration_spl = duration_str.split(":").collect::<Vec<&str>>();

        let inner = || {
            let duration_1st: u64 = duration_spl[0].parse()?;

            let duration_2nd: u64 = duration_spl[1].parse()?;

            let hours;
            let minutes;
            let seconds;

            if let Some(duration_3rd_str) = duration_spl.get(2) {
                let duration_3rd: u64 = duration_3rd_str.parse()?;

                hours = duration_1st;
                minutes = duration_2nd;
                seconds = duration_3rd;
            } else {
                hours = 0;
                minutes = duration_1st;
                seconds = duration_2nd;
            }

            Ok::<Duration, <u64 as FromStr>::Err>(Duration::from_secs(
                (hours * (60 * 60)) + (minutes * 60) + seconds,
            ))
        };

        if let Ok(new_duration) = inner() {
            *duration = new_duration;
        }
    }
}
