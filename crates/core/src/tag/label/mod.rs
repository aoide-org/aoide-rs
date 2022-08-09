// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::{Borrow, Cow},
    fmt,
    hash::Hash,
    ops::Deref,
};

use crate::{prelude::*, util::string::trimmed_non_empty_from};

pub type LabelValue = String;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
#[repr(transparent)]
pub struct CowLabel<'a>(Cow<'a, str>);

impl<'a> From<CowLabel<'a>> for Label {
    fn from(from: CowLabel<'a>) -> Self {
        Self(from.0.into())
    }
}

impl<'a> From<&'a Label> for CowLabel<'a> {
    fn from(from: &'a Label) -> Self {
        Self(from.0.as_str().into())
    }
}

impl<'a> AsRef<Cow<'a, str>> for CowLabel<'a> {
    fn as_ref(&self) -> &Cow<'a, str> {
        &self.0
    }
}

impl<'a> Deref for CowLabel<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Borrow<str> for CowLabel<'_> {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

/// The name of a tag.
///
/// Format: Unicode string without leading/trailing whitespace
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
#[repr(transparent)]
pub struct Label(LabelValue);

impl Label {
    pub fn clamp_value<'a>(value: impl Into<Cow<'a, str>>) -> Option<CowLabel<'a>> {
        trimmed_non_empty_from(value).map(CowLabel)
    }

    pub fn clamp_from<'a>(value: impl Into<Cow<'a, str>>) -> Option<Self> {
        Self::clamp_value(value).map(Into::into)
    }

    #[must_use]
    pub const fn new(value: LabelValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(&self) -> &LabelValue {
        let Self(value) = self;
        value
    }

    #[must_use]
    pub fn into_value(self) -> LabelValue {
        let Self(value) = self;
        value
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LabelInvalidity {
    Empty,
    Format,
}

impl Validate for Label {
    type Invalidity = LabelInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.value().is_empty(), Self::Invalidity::Empty)
            .invalidate_if(
                Self::clamp_value(self.value().as_str()) != Some(self.into()),
                Self::Invalidity::Format,
            )
            .into()
    }
}

impl From<LabelValue> for Label {
    fn from(value: LabelValue) -> Self {
        Self::new(value)
    }
}

impl From<Label> for LabelValue {
    fn from(from: Label) -> Self {
        from.into_value()
    }
}

impl AsRef<LabelValue> for Label {
    fn as_ref(&self) -> &LabelValue {
        let Self(value) = self;
        value
    }
}

impl Deref for Label {
    type Target = LabelValue;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

pub trait Labeled {
    fn label(&self) -> Option<&Label>;
}

impl Labeled for Label {
    fn label(&self) -> Option<&Self> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
