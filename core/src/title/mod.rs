// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Title {
    pub name: String,

    pub level: TitleLevel,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TitleInvalidity {
    NameEmpty,
}

impl Validate for Title {
    type Invalidity = TitleInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.name.trim().is_empty(), TitleInvalidity::NameEmpty)
            .into()
    }
}

#[derive(Debug)]
pub struct Titles;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TitlesInvalidity {
    Title(TitleInvalidity),
    MainTitleMissing,
    MainTitleAmbiguous,
}

pub const ANY_LEVEL_FILTER: Option<TitleLevel> = None;

pub const ANY_LANGUAGE_FILTER: Option<Option<&'static str>> = None;

impl Titles {
    pub fn validate<'a, I>(titles: I) -> ValidationResult<TitlesInvalidity>
    where
        I: Iterator<Item = &'a Title> + Clone,
    {
        let mut at_least_one_title = false;
        let mut context = titles
            .clone()
            .fold(ValidationContext::new(), |context, title| {
                at_least_one_title = true;
                context.validate_with(title, TitlesInvalidity::Title)
            });
        if context.is_valid() && at_least_one_title {
            context = match Self::main_titles(titles).count() {
                0 => context.invalidate(TitlesInvalidity::MainTitleMissing),
                1 => context, // ok
                _ => context.invalidate(TitlesInvalidity::MainTitleAmbiguous),
            }
        }
        context.into()
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
