// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_json::util::clock::DateTime;

use crate::prelude::*;

mod _inner {
    pub(super) use crate::_inner::filtering::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum FilterModifier {
    Complement,
}

#[cfg(feature = "backend")]
impl From<FilterModifier> for _inner::FilterModifier {
    fn from(from: FilterModifier) -> Self {
        use FilterModifier::*;
        match from {
            Complement => Self::Complement,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::FilterModifier> for FilterModifier {
    fn from(from: _inner::FilterModifier) -> Self {
        use _inner::FilterModifier::*;
        match from {
            Complement => Self::Complement,
        }
    }
}

/// Predicates for matching strings
#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum StringPredicate {
    StartsWith(String),
    StartsNotWith(String),
    EndsWith(String),
    EndsNotWith(String),
    Contains(String),
    ContainsNot(String),
    Matches(String),
    MatchesNot(String),
    Prefix(String),
    Equals(String),
    EqualsNot(String),
}

#[cfg(feature = "backend")]
impl From<StringPredicate> for _inner::StringPredicate {
    fn from(from: StringPredicate) -> Self {
        use StringPredicate::*;
        match from {
            StartsWith(s) => Self::StartsWith(s),
            StartsNotWith(s) => Self::StartsNotWith(s),
            EndsWith(s) => Self::EndsWith(s),
            EndsNotWith(s) => Self::EndsNotWith(s),
            Contains(s) => Self::Contains(s),
            ContainsNot(s) => Self::ContainsNot(s),
            Matches(s) => Self::Matches(s),
            MatchesNot(s) => Self::MatchesNot(s),
            Prefix(s) => Self::Prefix(s),
            Equals(s) => Self::Equals(s),
            EqualsNot(s) => Self::EqualsNot(s),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::StringPredicate> for StringPredicate {
    fn from(from: _inner::StringPredicate) -> Self {
        use _inner::StringPredicate::*;
        match from {
            StartsWith(s) => Self::StartsWith(s),
            StartsNotWith(s) => Self::StartsNotWith(s),
            EndsWith(s) => Self::EndsWith(s),
            EndsNotWith(s) => Self::EndsNotWith(s),
            Contains(s) => Self::Contains(s),
            ContainsNot(s) => Self::ContainsNot(s),
            Matches(s) => Self::Matches(s),
            MatchesNot(s) => Self::MatchesNot(s),
            Prefix(s) => Self::Prefix(s),
            Equals(s) => Self::Equals(s),
            EqualsNot(s) => Self::EqualsNot(s),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WithTokensQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<String>,
}

impl WithTokensQueryParams {
    #[must_use]
    pub fn try_with_token(&self, with_token: &str) -> bool {
        match self.with {
            Some(ref with) => with.split(',').any(|token| token == with_token),
            None => false,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<StringPredicate>,
}

#[cfg(feature = "backend")]
impl From<StringFilter> for _inner::StringFilter {
    fn from(from: StringFilter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            value: from.value.map(Into::into),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::StringFilter> for StringFilter {
    fn from(from: _inner::StringFilter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            value: from.value.map(Into::into),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum ScalarPredicate<V> {
    #[serde(rename = "lt")]
    LessThan(V),

    #[serde(rename = "le")]
    LessOrEqual(V),

    #[serde(rename = "gt")]
    GreaterThan(V),

    #[serde(rename = "ge")]
    GreaterOrEqual(V),

    #[serde(rename = "eq")]
    Equal(Option<V>),

    #[serde(rename = "ne")]
    NotEqual(Option<V>),
}

pub type NumericPredicate = ScalarPredicate<_inner::NumericValue>;

#[cfg(feature = "backend")]
impl From<NumericPredicate> for _inner::NumericPredicate {
    fn from(from: NumericPredicate) -> Self {
        use ScalarPredicate::*;
        match from {
            LessThan(val) => Self::LessThan(val),
            LessOrEqual(val) => Self::LessOrEqual(val),
            GreaterThan(val) => Self::GreaterThan(val),
            GreaterOrEqual(val) => Self::GreaterOrEqual(val),
            Equal(val) => Self::Equal(val),
            NotEqual(val) => Self::NotEqual(val),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::NumericPredicate> for NumericPredicate {
    fn from(from: _inner::NumericPredicate) -> Self {
        use _inner::ScalarPredicate::*;
        match from {
            LessThan(val) => Self::LessThan(val),
            LessOrEqual(val) => Self::LessOrEqual(val),
            GreaterThan(val) => Self::GreaterThan(val),
            GreaterOrEqual(val) => Self::GreaterOrEqual(val),
            Equal(val) => Self::Equal(val),
            NotEqual(val) => Self::NotEqual(val),
        }
    }
}

pub type DateTimePredicate = ScalarPredicate<DateTime>;

#[cfg(feature = "backend")]
impl From<DateTimePredicate> for _inner::DateTimePredicate {
    fn from(from: DateTimePredicate) -> Self {
        use ScalarPredicate::*;
        match from {
            LessThan(val) => Self::LessThan(val.into()),
            LessOrEqual(val) => Self::LessOrEqual(val.into()),
            GreaterThan(val) => Self::GreaterThan(val.into()),
            GreaterOrEqual(val) => Self::GreaterOrEqual(val.into()),
            Equal(val) => Self::Equal(val.map(Into::into)),
            NotEqual(val) => Self::NotEqual(val.map(Into::into)),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::DateTimePredicate> for DateTimePredicate {
    fn from(from: _inner::DateTimePredicate) -> Self {
        use _inner::ScalarPredicate::*;
        match from {
            LessThan(val) => Self::LessThan(val.into()),
            LessOrEqual(val) => Self::LessOrEqual(val.into()),
            GreaterThan(val) => Self::GreaterThan(val.into()),
            GreaterOrEqual(val) => Self::GreaterOrEqual(val.into()),
            Equal(val) => Self::Equal(val.map(Into::into)),
            NotEqual(val) => Self::NotEqual(val.map(Into::into)),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ScalarFieldFilter<F, V>(pub(crate) F, pub(crate) ScalarPredicate<V>);
