// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
        use FilterModifier as From;
        match from {
            From::Complement => Self::Complement,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::FilterModifier> for FilterModifier {
    fn from(from: _inner::FilterModifier) -> Self {
        use _inner::FilterModifier as From;
        match from {
            From::Complement => Self::Complement,
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
impl From<StringPredicate> for _inner::StringPredicate<'static> {
    fn from(from: StringPredicate) -> Self {
        use StringPredicate as From;
        match from {
            From::StartsWith(s) => Self::StartsWith(s.into()),
            From::StartsNotWith(s) => Self::StartsNotWith(s.into()),
            From::EndsWith(s) => Self::EndsWith(s.into()),
            From::EndsNotWith(s) => Self::EndsNotWith(s.into()),
            From::Contains(s) => Self::Contains(s.into()),
            From::ContainsNot(s) => Self::ContainsNot(s.into()),
            From::Matches(s) => Self::Matches(s.into()),
            From::MatchesNot(s) => Self::MatchesNot(s.into()),
            From::Prefix(s) => Self::Prefix(s.into()),
            From::Equals(s) => Self::Equals(s.into()),
            From::EqualsNot(s) => Self::EqualsNot(s.into()),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::StringPredicate<'static>> for StringPredicate {
    fn from(from: _inner::StringPredicate<'static>) -> Self {
        use _inner::StringPredicate as From;
        match from {
            From::StartsWith(s) => Self::StartsWith(s.into_owned()),
            From::StartsNotWith(s) => Self::StartsNotWith(s.into_owned()),
            From::EndsWith(s) => Self::EndsWith(s.into_owned()),
            From::EndsNotWith(s) => Self::EndsNotWith(s.into_owned()),
            From::Contains(s) => Self::Contains(s.into_owned()),
            From::ContainsNot(s) => Self::ContainsNot(s.into_owned()),
            From::Matches(s) => Self::Matches(s.into_owned()),
            From::MatchesNot(s) => Self::MatchesNot(s.into_owned()),
            From::Prefix(s) => Self::Prefix(s.into_owned()),
            From::Equals(s) => Self::Equals(s.into_owned()),
            From::EqualsNot(s) => Self::EqualsNot(s.into_owned()),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct StringFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<StringPredicate>,
}

#[cfg(feature = "backend")]
impl From<StringFilter> for _inner::StringFilter<'static> {
    fn from(from: StringFilter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            value: from.value.map(Into::into),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::StringFilter<'static>> for StringFilter {
    fn from(from: _inner::StringFilter<'static>) -> Self {
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
        use ScalarPredicate as From;
        match from {
            From::LessThan(val) => Self::LessThan(val),
            From::LessOrEqual(val) => Self::LessOrEqual(val),
            From::GreaterThan(val) => Self::GreaterThan(val),
            From::GreaterOrEqual(val) => Self::GreaterOrEqual(val),
            From::Equal(val) => Self::Equal(val),
            From::NotEqual(val) => Self::NotEqual(val),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::NumericPredicate> for NumericPredicate {
    fn from(from: _inner::NumericPredicate) -> Self {
        use _inner::ScalarPredicate as From;
        match from {
            From::LessThan(val) => Self::LessThan(val),
            From::LessOrEqual(val) => Self::LessOrEqual(val),
            From::GreaterThan(val) => Self::GreaterThan(val),
            From::GreaterOrEqual(val) => Self::GreaterOrEqual(val),
            From::Equal(val) => Self::Equal(val),
            From::NotEqual(val) => Self::NotEqual(val),
        }
    }
}

pub type DateTimePredicate = ScalarPredicate<DateTime>;

#[cfg(feature = "backend")]
impl From<DateTimePredicate> for _inner::DateTimePredicate {
    fn from(from: DateTimePredicate) -> Self {
        use ScalarPredicate as From;
        match from {
            From::LessThan(val) => Self::LessThan(val.into()),
            From::LessOrEqual(val) => Self::LessOrEqual(val.into()),
            From::GreaterThan(val) => Self::GreaterThan(val.into()),
            From::GreaterOrEqual(val) => Self::GreaterOrEqual(val.into()),
            From::Equal(val) => Self::Equal(val.map(Into::into)),
            From::NotEqual(val) => Self::NotEqual(val.map(Into::into)),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::DateTimePredicate> for DateTimePredicate {
    fn from(from: _inner::DateTimePredicate) -> Self {
        use _inner::ScalarPredicate as From;
        match from {
            From::LessThan(val) => Self::LessThan(val.into()),
            From::LessOrEqual(val) => Self::LessOrEqual(val.into()),
            From::GreaterThan(val) => Self::GreaterThan(val.into()),
            From::GreaterOrEqual(val) => Self::GreaterOrEqual(val.into()),
            From::Equal(val) => Self::Equal(val.map(Into::into)),
            From::NotEqual(val) => Self::NotEqual(val.map(Into::into)),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ScalarFieldFilter<F, V>(pub(crate) F, pub(crate) ScalarPredicate<V>);
