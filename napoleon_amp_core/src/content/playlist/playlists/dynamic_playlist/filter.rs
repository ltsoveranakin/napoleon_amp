use crate::content::song::Song;
use derive_enum_all_values::AllValues;
use serbytes::prelude::SerBytes;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub type FilterRules = FilterRulesTyped<FilterRule<String>, FilterRule<u8>>;

#[derive(SerBytes, AllValues, Debug, Copy, Clone)]
pub enum ComparisonMethod {
    LessThan,
    EqualTo,
    GreaterThan,
    Contains,
    NotEqualTo,
}

impl ComparisonMethod {
    pub fn get_display_str(&self) -> &str {
        match self {
            Self::LessThan => "Less than",
            Self::EqualTo => "Equal to",
            Self::GreaterThan => "Greater than",
            Self::Contains => "Contains",
            Self::NotEqualTo => "Not equal to",
        }
    }
}

impl Display for ComparisonMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_display_str())
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
    T: Ord + AsStr,
{
    fn does_value_pass(&self, test_value: &T) -> bool {
        match self.comparison_method {
            ComparisonMethod::LessThan => &self.value < test_value,
            ComparisonMethod::EqualTo => &self.value == test_value,
            ComparisonMethod::GreaterThan => &self.value > test_value,
            ComparisonMethod::NotEqualTo => &self.value != test_value,
            ComparisonMethod::Contains => {
                test_value
                    .as_cow_str()
                    .to_lowercase()
                    .contains(&self.value.as_cow_str().to_lowercase())
                // panic!("Cannot check contains on a non string test value")
            }
        }
    }
}

pub trait AsStr {
    fn as_cow_str(&self) -> Cow<'_, str>;
}

impl AsStr for String {
    fn as_cow_str(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
}

impl AsStr for u8 {
    fn as_cow_str(&self) -> Cow<'_, str> {
        Cow::Owned(self.to_string())
    }
}

#[derive(SerBytes, Debug, Clone)]
pub enum FilterRulesTyped<S, U> {
    Title(S),
    Artist(S),
    Album(S),
    Rating(U),
}

impl FilterRules {
    pub(super) fn does_song_pass(&self, song: &Song) -> bool {
        let song_data = &song.get_song_data().inner;

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

    pub fn try_assign_from_str(&mut self, s: &str) -> Result<(), ()> {
        match self {
            Self::Title(str_rule) | Self::Artist(str_rule) | Self::Album(str_rule) => {
                str_rule.value = s.to_string();
            }

            Self::Rating(rating) => {
                rating.value = u8::from_str(s).map_err(|_| ())?;
            }
        }

        Ok(())
    }

    pub fn from_variant(
        filter_rules: FilterRulesTyped<(), ()>,
        current_str_value: &str,
        cmp_method: ComparisonMethod,
    ) -> Self {
        match filter_rules {
            FilterRulesTyped::Title(_) => {
                Self::Title(FilterRule::new(current_str_value.to_string(), cmp_method))
            }
            FilterRulesTyped::Artist(_) => {
                Self::Artist(FilterRule::new(current_str_value.to_string(), cmp_method))
            }
            FilterRulesTyped::Album(_) => {
                Self::Album(FilterRule::new(current_str_value.to_string(), cmp_method))
            }
            FilterRulesTyped::Rating(_) => Self::Rating(FilterRule::new(
                u8::from_str(current_str_value).unwrap_or_default(),
                cmp_method,
            )),
        }
    }
}

impl<S, U> FilterRulesTyped<S, U> {
    pub fn get_display_str(&self) -> &'static str {
        match self {
            Self::Title(_) => "Title",
            Self::Artist(_) => "Artist",
            Self::Album(_) => "Album",
            Self::Rating(_) => "Rating",
        }
    }

    pub fn values() -> [FilterRulesTyped<(), ()>; 4] {
        [
            FilterRulesTyped::Title(()),
            FilterRulesTyped::Artist(()),
            FilterRulesTyped::Album(()),
            FilterRulesTyped::Rating(()),
        ]
    }
}

pub enum ValuesType<'a> {
    Str(&'a mut String),
    U8(&'a mut u8),
}
