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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

const NAME_MIN_LEN: usize = 1;

const LANG_MIN_LEN: usize = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Title {
    pub name: String,

    pub level: TitleLevel,

    pub language: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TitleValidation {
    NameMinLen(usize),
    LanguageMinLen(usize),
}

impl Validate for Title {
    type Validation = TitleValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(
            self.name.len() < NAME_MIN_LEN,
            TitleValidation::NameMinLen(NAME_MIN_LEN),
        );
        if let Some(ref language) = self.language {
            context.add_violation_if(
                language.len() < LANG_MIN_LEN,
                TitleValidation::LanguageMinLen(LANG_MIN_LEN),
            );
        }
        context.into_result()
    }
}

#[derive(Debug)]
pub struct Titles;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TitlesValidation {
    Title(TitleValidation),
    MainTitleMissing,
    MainTitleAmbiguous,
}

pub const ANY_LEVEL_FILTER: Option<TitleLevel> = None;

pub const ANY_LANGUAGE_FILTER: Option<Option<&'static str>> = None;

impl Titles {
    pub fn validate<'a, I>(titles: I) -> ValidationResult<TitlesValidation>
    where
        I: IntoIterator<Item = &'a Title> + Copy,
    {
        let mut context = ValidationContext::default();
        let mut at_least_one_title = false;
        for title in titles.into_iter() {
            context.map_and_merge_result(title.validate(), TitlesValidation::Title);
            at_least_one_title = true;
        }
        if !context.has_violations() && at_least_one_title {
            if Self::main_title(titles, None).is_none() {
                context.add_violation(TitlesValidation::MainTitleMissing);
            } else {
                let mut languages: Vec<Option<&'a str>> = titles
                    .into_iter()
                    .map(|title| title.language.as_ref().map(|s| s.as_str()))
                    .collect();
                languages.sort_unstable();
                languages.dedup();
                for language in &languages {
                    if Self::main_titles(titles, Some(*language)).count() > 1 {
                        context.add_violation(TitlesValidation::MainTitleAmbiguous);
                        break;
                    }
                }
            }
        }
        context.into_result()
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
