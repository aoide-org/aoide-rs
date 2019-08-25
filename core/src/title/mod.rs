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
    Main,
    Sub,
    // for classical music, only used for tracks not albums
    Work,
    Movement,
}

impl Default for TitleLevel {
    fn default() -> TitleLevel {
        TitleLevel::Main
    }
}

///////////////////////////////////////////////////////////////////////
// Title
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Title {
    pub name: String,

    pub level: TitleLevel,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TitleValidation {
    NameEmpty,
}

impl Validate for Title {
    type Validation = TitleValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(self.name.trim().is_empty(), TitleValidation::NameEmpty);
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
        I: Iterator<Item = &'a Title> + Clone,
    {
        let mut context = ValidationContext::default();
        let mut at_least_one_title = false;
        for title in titles.clone() {
            context.map_and_merge_result(title.validate(), TitlesValidation::Title);
            at_least_one_title = true;
        }
        if !context.has_violations() && at_least_one_title && Self::main_title(titles).is_none() {
            context.add_violation(TitlesValidation::MainTitleMissing);
        }
        context.into_result()
    }

    pub fn filter_level<'a, I>(
        titles: I,
        level: impl Into<Option<TitleLevel>>,
    ) -> impl Iterator<Item = &'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
    {
        let level = level.into();
        titles
            .into_iter()
            .filter(move |title| level == ANY_LEVEL_FILTER || level == Some(title.level))
    }

    pub fn main_titles<'a, 'b, I>(titles: I) -> impl Iterator<Item = &'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
    {
        Self::filter_level(titles, TitleLevel::Main)
    }

    pub fn main_title<'a, I>(titles: I) -> Option<&'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
    {
        Self::main_titles(titles).nth(0)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
