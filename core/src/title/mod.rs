// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::*;

///////////////////////////////////////////////////////////////////////
// TitleLevel
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TitleLevel {
    Main = 0, // default
    Sub = 1,
    // for classical music, only used for tracks not albums
    Work = 2,
    Movement = 3,
}

impl Default for TitleLevel {
    fn default() -> TitleLevel {
        TitleLevel::Main
    }
}

///////////////////////////////////////////////////////////////////////
// Title
///////////////////////////////////////////////////////////////////////

const MIN_NAME_LEN: usize = 1;

const MIN_LANG_LEN: usize = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Title {
    pub name: String,

    pub level: TitleLevel,

    pub language: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleValidation {
    Name,
    Language,
}

impl Validate<TitleValidation> for Title {
    fn validate(&self) -> ValidationResult<TitleValidation> {
        let mut errors = ValidationErrors::default();
        if self.name.len() < MIN_NAME_LEN {
            errors.add_error(
                TitleValidation::Name,
                Violation::TooShort(validate::Min(MIN_NAME_LEN)),
            );
        }
        if let Some(ref language) = self.language {
            if language.len() < MIN_LANG_LEN {
                errors.add_error(
                    TitleValidation::Name,
                    Violation::TooShort(validate::Min(MIN_LANG_LEN)),
                );
            }
        }
        errors.into_result()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Titles;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitlesValidation {
    Title(TitleValidation),
    LanguageIndependentMainTitle,
}

pub const ANY_LEVEL_FILTER: Option<TitleLevel> = None;

pub const ANY_LANGUAGE_FILTER: Option<Option<&'static str>> = None;

impl Titles {
    pub fn validate<'a, I>(titles: I) -> ValidationResult<TitlesValidation>
    where
        I: IntoIterator<Item = &'a Title> + Copy,
    {
        let mut errors = ValidationErrors::default();
        let mut at_least_one_title = false;
        for title in titles.into_iter() {
            errors.map_and_merge_result(title.validate(), TitlesValidation::Title);
            at_least_one_title = true;
        }
        if errors.is_empty() && at_least_one_title {
            if Self::main_title(titles, None).is_none() {
                errors.add_error(
                    TitlesValidation::LanguageIndependentMainTitle,
                    Violation::Missing,
                );
            } else {
                let mut languages: Vec<Option<&'a str>> = titles
                    .into_iter()
                    .map(|title| title.language.as_ref().map(|s| s.as_str()))
                    .collect();
                languages.sort();
                languages.dedup();
                for language in &languages {
                    if Self::main_titles(titles, Some(*language)).count() > 1 {
                        errors.add_error(
                            TitlesValidation::LanguageIndependentMainTitle,
                            Violation::TooMany(validate::Max(1)),
                        );
                        break;
                    }
                }
            }
        }
        errors.into_result()
    }

    pub fn filter_level_language<'a, 'b, I>(
        titles: I,
        level: impl Into<Option<TitleLevel>>,
        language: impl Into<Option<Option<&'b str>>>,
    ) -> impl Iterator<Item = &'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
        'b: 'a,
    {
        let level = level.into();
        let language = language.into();
        titles.into_iter().filter(move |title| {
            (level == ANY_LEVEL_FILTER || level == Some(title.level))
                && (language == ANY_LANGUAGE_FILTER
                    || language == Some(title.language.as_ref().map(String::as_str)))
        })
    }

    pub fn main_titles<'a, 'b, I>(
        titles: I,
        language: impl Into<Option<Option<&'b str>>>,
    ) -> impl Iterator<Item = &'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
        'b: 'a,
    {
        Self::filter_level_language(titles, TitleLevel::Main, language)
    }

    pub fn main_title<'a, 'b, I>(
        titles: I,
        default_language: impl Into<Option<&'b str>>,
    ) -> Option<&'a Title>
    where
        I: IntoIterator<Item = &'a Title> + Copy,
        'b: 'a,
    {
        if let Some(main_title) = Self::main_titles(titles, Some(None)).nth(0) {
            return Some(main_title);
        }
        let default_language = default_language.into();
        if default_language.is_some() {
            if let Some(main_title) = Self::main_titles(titles, Some(default_language)).nth(0) {
                return Some(main_title);
            }
        }
        Self::main_titles(titles, ANY_LANGUAGE_FILTER).nth(0)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
