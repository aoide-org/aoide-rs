// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::clock::DateTime;

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringPredicateBorrowed<'s> {
    // Case-insensitive comparison
    StartsWith(&'s str),
    StartsNotWith(&'s str),
    EndsWith(&'s str),
    EndsNotWith(&'s str),
    Contains(&'s str),
    ContainsNot(&'s str),
    Matches(&'s str),
    MatchesNot(&'s str),
    // Case-sensitive comparison
    Prefix(&'s str),
    Equals(&'s str),
    EqualsNot(&'s str),
}

/// Predicates for matching strings
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StringPredicate {
    // Case-insensitive comparison
    StartsWith(String),
    StartsNotWith(String),
    EndsWith(String),
    EndsNotWith(String),
    Contains(String),
    ContainsNot(String),
    Matches(String),
    MatchesNot(String),
    // Case-sensitive comparison
    Prefix(String),
    Equals(String),
    EqualsNot(String),
}

impl StringPredicate {
    #[must_use]
    pub fn borrow(&self) -> StringPredicateBorrowed<'_> {
        match self {
            Self::StartsWith(s) => StringPredicateBorrowed::StartsWith(s.as_str()),
            Self::StartsNotWith(s) => StringPredicateBorrowed::StartsNotWith(s.as_str()),
            Self::EndsWith(s) => StringPredicateBorrowed::EndsWith(s.as_str()),
            Self::EndsNotWith(s) => StringPredicateBorrowed::EndsNotWith(s.as_str()),
            Self::Contains(s) => StringPredicateBorrowed::Contains(s.as_str()),
            Self::ContainsNot(s) => StringPredicateBorrowed::ContainsNot(s.as_str()),
            Self::Matches(s) => StringPredicateBorrowed::Matches(s.as_str()),
            Self::MatchesNot(s) => StringPredicateBorrowed::MatchesNot(s.as_str()),
            Self::Prefix(s) => StringPredicateBorrowed::Prefix(s.as_str()),
            Self::Equals(s) => StringPredicateBorrowed::Equals(s.as_str()),
            Self::EqualsNot(s) => StringPredicateBorrowed::EqualsNot(s.as_str()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StringFilterBorrowed<'s> {
    pub modifier: Option<FilterModifier>,
    pub value: Option<StringPredicateBorrowed<'s>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StringFilter {
    pub modifier: Option<FilterModifier>,
    pub value: Option<StringPredicate>,
}

impl StringFilter {
    #[must_use]
    pub fn borrow(&self) -> StringFilterBorrowed<'_> {
        let Self { modifier, value } = self;
        StringFilterBorrowed {
            modifier: *modifier,
            value: value.as_ref().map(StringPredicate::borrow),
        }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScalarFieldFilter<F, V> {
    pub field: F,
    pub predicate: ScalarPredicate<V>,
}

pub type NumericValue = f64;

pub type NumericPredicate = ScalarPredicate<NumericValue>;

pub type DateTimePredicate = ScalarPredicate<DateTime>;
