// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    fmt,
    hash::Hash,
    ops::{Deref, Not as _},
};

use crate::{prelude::*, util::canonical::CanonicalOrd};

pub type FacetIdValue = String;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
#[repr(transparent)]
pub struct CowFacetId<'a>(Cow<'a, str>);

impl<'a> From<CowFacetId<'a>> for FacetId {
    fn from(from: CowFacetId<'a>) -> Self {
        Self(from.0.into())
    }
}

impl<'a> From<&'a FacetId> for CowFacetId<'a> {
    fn from(from: &'a FacetId) -> Self {
        Self(from.0.as_str().into())
    }
}

impl<'a> AsRef<Cow<'a, str>> for CowFacetId<'a> {
    fn as_ref(&self) -> &Cow<'a, str> {
        &self.0
    }
}

impl<'a> Deref for CowFacetId<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Borrow<str> for CowFacetId<'_> {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

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
///   - starts with an ASCII lowercase character or '~'
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
pub struct FacetId(FacetIdValue);

/// The alphabet of facet identifiers
///
/// All valid characters, ordered by their ASCII codes.
pub const FACET_ID_ALPHABET: &str = "+-./0123456789@[]_abcdefghijklmnopqrstuvwxyz~";

impl FacetId {
    pub fn clamp_value<'a>(value: impl Into<Cow<'a, str>>) -> Option<CowFacetId<'a>> {
        let mut value: String = value.into().into();
        value.retain(Self::is_valid_char);
        if value.is_empty() {
            None
        } else {
            Some(CowFacetId(Cow::Owned(value)))
        }
    }

    pub fn clamp_from<'a>(value: impl Into<Cow<'a, str>>) -> Option<Self> {
        Self::clamp_value(value).map(Into::into)
    }

    #[must_use]
    pub const fn new(value: FacetIdValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn into_value(self) -> FacetIdValue {
        let Self(value) = self;
        value
    }

    #[must_use]
    pub const fn value(&self) -> &FacetIdValue {
        let Self(value) = self;
        value
    }

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

    fn is_invalid_char(c: char) -> bool {
        !Self::is_valid_char(c)
    }
}

impl CanonicalOrd for FacetId {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FacetIdInvalidity {
    Empty,
    Format,
}

impl Validate for FacetId {
    type Invalidity = FacetIdInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.value().is_empty(), Self::Invalidity::Empty)
            .invalidate_if(
                self.value()
                    .chars()
                    .next()
                    .map_or(false, |c| (c.is_ascii_lowercase() || c == '~').not()),
                Self::Invalidity::Format,
            )
            .invalidate_if(
                self.value().chars().any(FacetId::is_invalid_char),
                Self::Invalidity::Format,
            )
            .into()
    }
}

impl From<FacetIdValue> for FacetId {
    fn from(from: FacetIdValue) -> Self {
        Self::new(from)
    }
}

impl From<FacetId> for FacetIdValue {
    fn from(from: FacetId) -> Self {
        from.into_value()
    }
}

impl AsRef<FacetIdValue> for FacetId {
    fn as_ref(&self) -> &FacetIdValue {
        let Self(value) = self;
        value
    }
}

impl Deref for FacetId {
    type Target = FacetIdValue;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<str> for FacetId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for FacetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.value())
    }
}

pub trait Faceted {
    fn facet(&self) -> Option<&FacetId>;
}

impl Faceted for FacetId {
    fn facet(&self) -> Option<&Self> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
