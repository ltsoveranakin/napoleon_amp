use crate::content::song::Song;
use serbytes::prelude::SerBytes;
use std::fmt::{Display, Formatter};

#[derive(SerBytes, Debug, Copy, Clone)]
pub enum ComparisonMethod {
    LessThan,
    EqualTo,
    GreaterThan,
    Contains,
    NotEqualTo,
}

impl Display for ComparisonMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::LessThan => "Less than",
            Self::EqualTo => "Equal to",
            Self::GreaterThan => "Greater than",
            Self::Contains => "Contains",
            Self::NotEqualTo => "Not equal to",
        };

        f.write_str(s)
    }
}

#[derive(SerBytes, Debug, Copy, Clone)]
pub struct FilterRule<T> {
    pub value: T,
    pub comparison_method: ComparisonMethod,
}

impl<T> FilterRule<T> {
    pub fn new(value: T, comparison_method: ComparisonMethod) -> Self {
        Self {
            value,
            comparison_method,
        }
    }
}

impl<T> FilterRule<T>
where
    T: Ord,
{
    fn does_value_pass(&self, test_value: &T) -> bool {
        match self.comparison_method {
            ComparisonMethod::LessThan => &self.value < test_value,
            ComparisonMethod::EqualTo => &self.value == test_value,
            ComparisonMethod::GreaterThan => &self.value > test_value,
            ComparisonMethod::NotEqualTo => &self.value != test_value,
            ComparisonMethod::Contains => {
                panic!("Cannot check contains on a non string test value")
            }
        }
    }
}

impl FilterRule<String> {
    fn does_str_pass(&self, test_str: &String) -> bool {
        match self.comparison_method {
            ComparisonMethod::LessThan
            | ComparisonMethod::EqualTo
            | ComparisonMethod::GreaterThan
            | ComparisonMethod::NotEqualTo => self.does_value_pass(test_str),

            ComparisonMethod::Contains => self.value.contains(test_str),
        }
    }
}

#[derive(SerBytes, Debug, Clone)]
pub enum FilterRules {
    Title(FilterRule<String>),
    Artist(FilterRule<String>),
    Album(FilterRule<String>),
    Rating(FilterRule<u8>),
}

impl FilterRules {
    pub(super) fn does_song_pass(&self, song: &Song) -> bool {
        let song_data = &**song.get_song_data();

        match self {
            Self::Title(title) => title.does_value_pass(&song_data.title),
            Self::Artist(artist) => artist.does_value_pass(&song_data.artist.full_artist_string),
            Self::Album(album) => album.does_value_pass(&song_data.album),
            Self::Rating(rating) => rating.does_value_pass(&song_data.rating),
        }
    }

    pub fn get_mut_values_pair(&mut self) -> (ValuesType<'_>, &mut ComparisonMethod) {
        match self {
            Self::Title(title) => (
                ValuesType::Str(&mut title.value),
                &mut title.comparison_method,
            ),
            Self::Artist(artist) => (
                ValuesType::Str(&mut artist.value),
                &mut artist.comparison_method,
            ),
            Self::Album(album) => (
                ValuesType::Str(&mut album.value),
                &mut album.comparison_method,
            ),
            Self::Rating(rating) => (
                ValuesType::U8(&mut rating.value),
                &mut rating.comparison_method,
            ),
        }
    }
}

pub enum ValuesType<'a> {
    Str(&'a mut String),
    U8(&'a mut u8),
}
