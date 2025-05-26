// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    hash::Hash,
    ops::Not as _,
};

use derive_more::Display;
use nonicle::CanonicalOrd;
use semval::prelude::*;
use smol_str::SmolStr;

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
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct FacetId(SmolStr);

#[cfg(feature = "json-schema")]
impl schemars::JsonSchema for FacetId {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("FacetId")
    }

    fn json_schema(schema_gen: &mut schemars::generate::SchemaGenerator) -> schemars::Schema {
        let mut schema = schema_gen.subschema_for::<String>();
        let schema_object = schema.ensure_object();
        schema_object.insert("title".to_owned(), "Tag facet identifier string".into());
        schema_object.insert(
            "description".to_owned(),
            format!("Only the following characters are allowed: \"{FACET_ID_ALPHABET}\"").into(),
        );
        schema_object.insert(
            "examples".to_owned(),
            vec![
                serde_json::to_value(crate::track::tag::FACET_ID_GENRE).expect("valid"),
                serde_json::to_value(crate::track::tag::FACET_ID_MBID_RECORDING).expect("valid"),
                serde_json::to_value(crate::track::tag::FACET_ID_VALENCE).expect("valid"),
            ]
            .into(),
        );
        schema
    }
}

/// The alphabet of facet identifiers
///
/// All valid characters, ordered by their ASCII codes.
pub const FACET_ID_ALPHABET: &str = "+-./0123456789@[]_abcdefghijklmnopqrstuvwxyz~";

impl FacetId {
    #[must_use]
    fn is_valid_char(c: char) -> bool {
        FACET_ID_ALPHABET.contains(c)
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
    fn clamp_inner(inner: SmolStr) -> Option<SmolStr> {
        if inner.is_empty() {
            return None;
        }
        if Self::is_valid_format(&inner) {
            return Some(inner);
        }
        let mut clamped = String::from(inner);
        clamped.retain(Self::is_valid_char);
        clamped.is_empty().not().then(|| clamped.into())
    }

    pub fn clamp_from(from: impl Into<SmolStr>) -> Option<Self> {
        let clamped = Self::clamp_inner(from.into()).map(Self::new_unchecked)?;
        debug_assert!(clamped.is_valid());
        Some(clamped)
    }

    #[must_use]
    pub fn from_unchecked(from: impl Into<SmolStr>) -> Self {
        let inner = from.into();
        let unchecked = Self::new_unchecked(inner);
        debug_assert!(unchecked.is_valid());
        unchecked
    }

    #[must_use]
    pub const fn new_unchecked(inner: SmolStr) -> Self {
        Self(inner)
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
}

impl Borrow<str> for FacetId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for FacetId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<SmolStr> for FacetId {
    fn as_ref(&self) -> &SmolStr {
        &self.0
    }
}

impl CanonicalOrd for FacetId {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl From<FacetId> for SmolStr {
    fn from(from: FacetId) -> Self {
        let FacetId(inner) = from;
        inner
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
            .invalidate_if(self.is_empty(), Self::Invalidity::Empty)
            .invalidate_if(
                Self::is_invalid_format(self.as_ref()),
                Self::Invalidity::Format,
            )
            .into()
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
