// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::{cmp::Ordering, iter::once};

use num_derive::{FromPrimitive, ToPrimitive};

use crate::{
    prelude::*,
    util::canonical::{CanonicalOrd, Canonicalize, IsCanonical},
};

///////////////////////////////////////////////////////////////////////
// TitleKind
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
pub enum TitleKind {
    Main = 0,
    Sub = 1,
    Sorting = 2,
    // for classical music, only used for tracks not albums
    Work = 3,
    Movement = 4,
}

impl Default for TitleKind {
    fn default() -> TitleKind {
        TitleKind::Main
    }
}

///////////////////////////////////////////////////////////////////////
// Title
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Title {
    pub kind: TitleKind,

    pub name: String,
}

impl CanonicalOrd for Title {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        let Self {
            kind: lhs_kind,
            name: lhs_name,
        } = self;
        let Self {
            kind: rhs_kind,
            name: rhs_name,
        } = other;
        lhs_kind.cmp(rhs_kind).then(lhs_name.cmp(rhs_name))
    }
}

impl IsCanonical for Title {
    fn is_canonical(&self) -> bool {
        true
    }
}

impl Canonicalize for Title {
    fn canonicalize(&mut self) {
        debug_assert!(self.is_canonical());
    }
}

pub fn is_valid_title_name(name: impl AsRef<str>) -> bool {
    let name = name.as_ref();
    let trimmed = name.trim();
    !trimmed.is_empty() && trimmed == name
}

#[derive(Copy, Clone, Debug)]
pub enum TitleInvalidity {
    Name,
}

impl Validate for Title {
    type Invalidity = TitleInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!is_valid_title_name(&self.name), Self::Invalidity::Name)
            .into()
    }
}

#[derive(Debug)]
pub struct Titles;

#[derive(Copy, Clone, Debug)]
pub enum TitlesInvalidity {
    Title(TitleInvalidity),
    MainTitleMissing,
    MainTitleAmbiguous,
}

pub const ANY_KIND_FILTER: Option<TitleKind> = None;

pub const ANY_LANGUAGE_FILTER: Option<Option<&'static str>> = None;

impl Titles {
    pub fn validate<'a, I>(titles: &I) -> ValidationResult<TitlesInvalidity>
    where
        I: Iterator<Item = &'a Title> + Clone,
    {
        let mut at_least_one_title = false;
        let mut context = titles
            .to_owned()
            .fold(ValidationContext::new(), |context, title| {
                at_least_one_title = true;
                context.validate_with(title, TitlesInvalidity::Title)
            });
        if context.is_valid() && at_least_one_title {
            context = match Self::main_titles(titles.to_owned()).count() {
                0 => context.invalidate(TitlesInvalidity::MainTitleMissing),
                1 => context, // ok
                _ => context.invalidate(TitlesInvalidity::MainTitleAmbiguous),
            }
        }
        context.into()
    }

    pub fn filter_kind<'a, I>(
        titles: I,
        kind: impl Into<Option<TitleKind>>,
    ) -> impl Iterator<Item = &'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
    {
        let kind = kind.into();
        titles
            .into_iter()
            .filter(move |title| kind == ANY_KIND_FILTER || kind == Some(title.kind))
    }

    pub fn main_titles<'a, 'b, I>(titles: I) -> impl Iterator<Item = &'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
    {
        Self::filter_kind(titles, TitleKind::Main)
    }

    pub fn main_title<'a, I>(titles: I) -> Option<&'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
    {
        Self::main_titles(titles).next()
    }

    pub fn set_main_title(titles: &mut Vec<Title>, name: impl Into<String>) -> bool {
        let name = name.into();
        if let Some(main_title) = Self::main_title(titles.iter()) {
            // Replace
            if main_title.name == name {
                return false; // unmodified
            }
            let kind = main_title.kind;
            let old_titles = std::mem::take(titles);
            let new_titles = once(Title { kind, name })
                .chain(old_titles.into_iter().filter(|title| title.kind != kind))
                .collect();
            let placeholder = std::mem::replace(titles, new_titles);
            debug_assert!(placeholder.is_empty());
        } else {
            // Add
            titles.push(Title {
                name,
                kind: TitleKind::Main,
            });
        }
        true // modified
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
