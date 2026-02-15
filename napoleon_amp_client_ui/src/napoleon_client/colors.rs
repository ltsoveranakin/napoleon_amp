use eframe::egui::Color32;
use std::ops::Add;

pub(crate) const DEFAULT_TEXT_COLOR: Color32 = Color32::PLACEHOLDER;

pub(crate) const SELECTED_TEXT_COLOR: Color32 = Color32::from_rgb(67, 64, 237);
pub(crate) const SONG_PLAYING_TEXT_COLOR: Color32 = Color32::from_rgb(222, 64, 2);

pub(crate) trait Average {
    fn average(self, other: Self) -> Self;

    fn average_assign(&mut self, other: Self)
    where
        Self: Sized + Copy,
    {
        *self = self.average(other);
    }
}

impl Average for Color32 {
    fn average(self, other: Self) -> Self {
        let mut col = self.add(other);

        for i in 0..3 {
            col[i] /= 2;
        }

        col
    }
}
