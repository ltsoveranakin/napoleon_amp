use crate::content::song::Song;
use serbytes::prelude::SerBytes;

#[derive(SerBytes, Debug)]
pub enum ComparisonMethod {
    LessThan,
    EqualTo,
    GreaterThan,
    Contains,
    NotEqualTo,
}

#[derive(SerBytes, Debug)]
pub struct FilterRule<T> {
    value: T,
    comparison_method: ComparisonMethod,
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

#[derive(SerBytes, Debug)]
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
}
