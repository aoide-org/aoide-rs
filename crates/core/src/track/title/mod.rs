// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{cmp::Ordering, iter::once};

use nonicle::{CanonicalOrd, Canonicalize, IsCanonical};
use semval::prelude::*;
use strum::FromRepr;

///////////////////////////////////////////////////////////////////////
// Kind
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, FromRepr)]
#[repr(u8)]
pub enum Kind {
    #[default]
    Main = 0,
    Sub = 1,
    Sorting = 2,
    // for classical music, only used for tracks not albums
    Work = 3,
    Movement = 4,
}

///////////////////////////////////////////////////////////////////////
// Title
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Title {
    pub kind: Kind,

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
    SortTitleAmbiguous,
}

pub const ANY_KIND_FILTER: Option<Kind> = None;

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
            context = match Self::filter_kind(titles.to_owned(), Kind::Main).count() {
                0 => context.invalidate(TitlesInvalidity::MainTitleMissing),
                1 => context, // ok
                _ => context.invalidate(TitlesInvalidity::MainTitleAmbiguous),
            }
        }
        if context.is_valid() {
            context = match Self::filter_kind(titles.to_owned(), Kind::Sorting).count() {
                0 | 1 => context, // ok
                _ => context.invalidate(TitlesInvalidity::SortTitleAmbiguous),
            }
        }
        context.into()
    }

    pub fn filter_kind<'a, I>(
        titles: I,
        kind: impl Into<Option<Kind>>,
    ) -> impl Iterator<Item = &'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
    {
        let kind = kind.into();
        titles
            .into_iter()
            .filter(move |title| kind == ANY_KIND_FILTER || kind == Some(title.kind))
    }

    pub fn kind_title<'a, I>(titles: I, kind: Kind) -> Option<&'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
    {
        let mut iter = Self::filter_kind(titles, kind);
        let first = iter.next();
        debug_assert!(first.is_none() || iter.next().is_none());
        first
    }

    #[must_use]
    pub fn main_title<'a, I>(titles: I) -> Option<&'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
    {
        Self::kind_title(titles, Kind::Main)
    }

    #[must_use]
    pub fn sort_title<'a, 'b, I>(titles: I) -> Option<&'a Title>
    where
        I: IntoIterator<Item = &'a Title>,
    {
        Self::kind_title(titles, Kind::Sorting)
    }

    #[must_use]
    pub fn sort_or_main_title<'a, 'b, I>(titles: I) -> Option<&'a Title>
    where
        I: IntoIterator<Item = &'a Title> + Clone,
    {
        Self::sort_title(titles.clone()).or_else(|| Self::main_title(titles))
    }

    pub fn set_main_title(titles: &mut Vec<Title>, name: impl Into<String>) -> bool {
        debug_assert!(titles.is_canonical());
        let name = name.into();
        if let Some(main_title) = Self::main_title(titles.iter()) {
            // Replace
            if main_title.name == name {
                // Unmodified (and still canonical)
                debug_assert!(titles.is_canonical());
                return false;
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
                kind: Kind::Main,
            });
        }
        // Modified (and probably no longer canonical)
        true
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
