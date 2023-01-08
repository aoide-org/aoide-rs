// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    fmt,
    hash::Hash,
};

use crate::{prelude::*, util::canonical::CanonicalOrd};

/// An identifier for referencing tag categories.
///
/// Facets are used for grouping/categorizing and providing context or meaning.
///
/// Serves as a symbolic, internal identifier that is not intended to be displayed
/// literally in the UI. The restrictive naming constraints ensure that they are
/// not used for storing arbitrary text. Instead facet identifiers should be mapped
/// to translated display strings, e.g. the facet "genre" could be mapped to "Genre"
/// in English and the facet "venue" could be mapped to "Veranstaltungsort" in German.
///
/// Value constraints:
///   - charset/alphabet: `+-./0123456789@[]_abcdefghijklmnopqrstuvwxyz~`
///   - no leading/trailing/inner whitespace
///
/// Rationale for the value constraints:
///   - Facet identifiers are intended to be created, shared, and parsed worldwide
///   - The Lingua franca of IT is English
///   - ASCII characters can be encoded by a single byte in UTF-8
///
/// References:
///   - <https://en.wikipedia.org/wiki/Faceted_classification>
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
#[repr(transparent)]
pub struct FacetId<'a>(Cow<'a, str>);

/// The alphabet of facet identifiers
///
/// All valid characters, ordered by their ASCII codes.
pub const FACET_ID_ALPHABET: &str = "+-./0123456789@[]_abcdefghijklmnopqrstuvwxyz~";

impl<'a> FacetId<'a> {
    #[must_use]
    fn is_valid_char(c: char) -> bool {
        // TODO: Use regex?
        if !c.is_ascii() || c.is_ascii_whitespace() || c.is_ascii_uppercase() {
            return false;
        }
        if c.is_ascii_alphanumeric() {
            return true;
        }
        "+-./@[]_~".contains(c)
    }

    #[must_use]
    fn is_invalid_char(c: char) -> bool {
        !Self::is_valid_char(c)
    }

    #[must_use]
    fn is_invalid_format(inner: &str) -> bool {
        inner.contains(Self::is_invalid_char)
    }

    #[must_use]
    fn is_valid_format(inner: &str) -> bool {
        !Self::is_invalid_format(inner)
    }

    #[must_use]
    fn clamp_inner(inner: Cow<'a, str>) -> Option<Cow<'a, str>> {
        if inner.is_empty() {
            return None;
        }
        if Self::is_valid_format(&inner) {
            return Some(inner);
        }
        if !inner.contains(Self::is_valid_char) {
            return None;
        }
        let mut owned = inner.into_owned();
        owned.retain(Self::is_valid_char);
        debug_assert!(!owned.is_empty());
        Some(Cow::Owned(owned))
    }

    pub fn clamp_from(from: impl Into<Cow<'a, str>>) -> Option<FacetId<'a>> {
        Self::clamp_inner(from.into()).map(Self::new)
    }

    #[must_use]
    pub const fn new(inner: Cow<'a, str>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn into_inner(self) -> Cow<'a, str> {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    #[must_use]
    pub fn as_borrowed(&'a self) -> Self {
        let Self(inner) = self;
        FacetId(Cow::Borrowed(inner))
    }

    #[must_use]
    pub fn into_owned(self) -> FacetId<'static> {
        let Self(inner) = self;
        FacetId(Cow::Owned(inner.into_owned()))
    }
}

impl<'a> AsRef<Cow<'a, str>> for FacetId<'a> {
    fn as_ref(&self) -> &Cow<'a, str> {
        &self.0
    }
}

impl Borrow<str> for FacetId<'_> {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for FacetId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl CanonicalOrd for FacetId<'_> {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FacetIdInvalidity {
    Empty,
    Format,
}

impl Validate for FacetId<'_> {
    type Invalidity = FacetIdInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.is_empty(), Self::Invalidity::Empty)
            .invalidate_if(
                Self::is_invalid_format(self.as_ref()),
                Self::Invalidity::Format,
            )
            .into()
    }
}

pub trait Faceted {
    fn facet(&self) -> Option<&FacetId<'_>>;
}

impl Faceted for FacetId<'_> {
    fn facet(&self) -> Option<&Self> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
