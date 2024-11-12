// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    fmt,
    hash::Hash,
};

use nonicle::CanonicalOrd;
use semval::prelude::*;

use crate::util::string::trimmed_non_empty_from;

/// The name of a tag.
///
/// Format: Unicode string without leading/trailing whitespace
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
pub struct Label<'a>(Cow<'a, str>);

impl<'a> Label<'a> {
    #[must_use]
    fn is_invalid_format(inner: &str) -> bool {
        !Self::is_valid_format(inner)
    }

    #[must_use]
    fn is_valid_format(inner: &str) -> bool {
        inner.trim() == inner
    }

    #[must_use]
    fn clamp_inner(inner: Cow<'a, str>) -> Option<Cow<'a, str>> {
        trimmed_non_empty_from(inner)
    }

    pub fn clamp_from(from: impl Into<Cow<'a, str>>) -> Option<Self> {
        let clamped = Self::clamp_inner(from.into()).map(Self::new);
        debug_assert!(clamped.is_valid());
        clamped
    }

    #[must_use]
    pub fn from_unchecked(from: impl Into<Cow<'a, str>>) -> Self {
        let inner = from.into();
        let unchecked = Self::new(inner);
        debug_assert!(unchecked.is_valid());
        unchecked
    }

    #[must_use]
    pub const fn new(inner: Cow<'a, str>) -> Self {
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

    #[must_use]
    pub fn to_borrowed(&'a self) -> Self {
        let Self(inner) = self;
        Label(Cow::Borrowed(inner))
    }

    #[must_use]
    pub fn into_owned(self) -> Label<'static> {
        let Self(inner) = self;
        Label(Cow::Owned(inner.into_owned()))
    }

    #[must_use]
    pub fn clone_owned(&self) -> Label<'static> {
        self.to_borrowed().into_owned()
    }
}

impl<'a> AsRef<Cow<'a, str>> for Label<'a> {
    fn as_ref(&self) -> &Cow<'a, str> {
        &self.0
    }
}

impl Borrow<str> for Label<'_> {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for Label<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl CanonicalOrd for Label<'_> {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl<'a> From<Label<'a>> for Cow<'a, str> {
    fn from(from: Label<'a>) -> Self {
        let Label(inner) = from;
        inner
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LabelInvalidity {
    Empty,
    Format,
}

impl Validate for Label<'_> {
    type Invalidity = LabelInvalidity;

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

pub trait Labeled {
    fn label(&self) -> Option<&Label<'_>>;
}

impl Labeled for Label<'_> {
    fn label(&self) -> Option<&Self> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
