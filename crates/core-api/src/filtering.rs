// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use aoide_core::util::clock::OffsetDateTimeMs;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FilterModifier {
    Complement,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StringCompare {
    StartsWith, // head (case-insensitive)
    EndsWith,   // tail (case-insensitive)
    Contains,   // part (case-insensitive)
    Matches,    // all (case-insensitive)
    Prefix,     // head (case-sensitive)
    Equals,     // all (case-sensitive)
}

/// Predicates for matching strings
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StringPredicate<'s> {
    // Case-insensitive comparison
    StartsWith(Cow<'s, str>),
    StartsNotWith(Cow<'s, str>),
    EndsWith(Cow<'s, str>),
    EndsNotWith(Cow<'s, str>),
    Contains(Cow<'s, str>),
    ContainsNot(Cow<'s, str>),
    Matches(Cow<'s, str>),
    MatchesNot(Cow<'s, str>),
    // Case-sensitive comparison
    Equals(Cow<'s, str>),
    EqualsNot(Cow<'s, str>),
    Prefix(Cow<'s, str>),
}

impl<'s> StringPredicate<'s> {
    #[must_use]
    pub fn to_borrowed(&'s self) -> Self {
        match self {
            Self::StartsWith(inner) => Self::StartsWith(Cow::Borrowed(inner)),
            Self::StartsNotWith(inner) => Self::StartsNotWith(Cow::Borrowed(inner)),
            Self::EndsWith(inner) => Self::EndsWith(Cow::Borrowed(inner)),
            Self::EndsNotWith(inner) => Self::EndsNotWith(Cow::Borrowed(inner)),
            Self::Contains(inner) => Self::Contains(Cow::Borrowed(inner)),
            Self::ContainsNot(inner) => Self::ContainsNot(Cow::Borrowed(inner)),
            Self::Matches(inner) => Self::Matches(Cow::Borrowed(inner)),
            Self::MatchesNot(inner) => Self::MatchesNot(Cow::Borrowed(inner)),
            Self::Equals(inner) => Self::Equals(Cow::Borrowed(inner)),
            Self::EqualsNot(inner) => Self::EqualsNot(Cow::Borrowed(inner)),
            Self::Prefix(inner) => Self::Prefix(Cow::Borrowed(inner)),
        }
    }

    #[must_use]
    pub fn into_owned(self) -> StringPredicate<'static> {
        match self {
            Self::StartsWith(inner) => StringPredicate::StartsWith(Cow::Owned(inner.into_owned())),
            Self::StartsNotWith(inner) => {
                StringPredicate::StartsNotWith(Cow::Owned(inner.into_owned()))
            }
            Self::EndsWith(inner) => StringPredicate::EndsWith(Cow::Owned(inner.into_owned())),
            Self::EndsNotWith(inner) => {
                StringPredicate::EndsNotWith(Cow::Owned(inner.into_owned()))
            }
            Self::Contains(inner) => StringPredicate::Contains(Cow::Owned(inner.into_owned())),
            Self::ContainsNot(inner) => {
                StringPredicate::ContainsNot(Cow::Owned(inner.into_owned()))
            }
            Self::Matches(inner) => StringPredicate::Matches(Cow::Owned(inner.into_owned())),
            Self::MatchesNot(inner) => StringPredicate::MatchesNot(Cow::Owned(inner.into_owned())),
            Self::Equals(inner) => StringPredicate::Equals(Cow::Owned(inner.into_owned())),
            Self::EqualsNot(inner) => StringPredicate::EqualsNot(Cow::Owned(inner.into_owned())),
            Self::Prefix(inner) => StringPredicate::Prefix(Cow::Owned(inner.into_owned())),
        }
    }

    #[must_use]
    pub fn clone_owned(&self) -> StringPredicate<'static> {
        self.to_borrowed().into_owned()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StringFilter<'s> {
    pub modifier: Option<FilterModifier>,
    pub value: Option<StringPredicate<'s>>,
}

impl<'s> StringFilter<'s> {
    #[must_use]
    pub fn to_borrowed(&'s self) -> Self {
        let Self { modifier, value } = self;
        Self {
            modifier: *modifier,
            value: value.as_ref().map(StringPredicate::to_borrowed),
        }
    }

    #[must_use]
    pub fn into_owned(self) -> StringFilter<'static> {
        let Self { modifier, value } = self;
        StringFilter {
            modifier,
            value: value.map(StringPredicate::into_owned),
        }
    }

    #[must_use]
    pub fn clone_owned(&self) -> StringFilter<'static> {
        self.to_borrowed().into_owned()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ScalarPredicate<V> {
    LessThan(V),
    LessOrEqual(V),
    GreaterThan(V),
    GreaterOrEqual(V),
    Equal(Option<V>),    // nullable
    NotEqual(Option<V>), // nullable
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScalarFieldFilter<F, V> {
    pub field: F,
    pub predicate: ScalarPredicate<V>,
}

pub type NumericValue = f64;

pub type NumericPredicate = ScalarPredicate<NumericValue>;

pub type DateTimePredicate = ScalarPredicate<OffsetDateTimeMs>;
