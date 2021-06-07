// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::prelude::*;

mod _core {
    pub use aoide_core::usecases::filtering::*;
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FilterModifier {
    Complement,
}

impl From<FilterModifier> for _core::FilterModifier {
    fn from(from: FilterModifier) -> Self {
        use FilterModifier::*;
        match from {
            Complement => Self::Complement,
        }
    }
}

impl From<_core::FilterModifier> for FilterModifier {
    fn from(from: _core::FilterModifier) -> Self {
        use _core::FilterModifier::*;
        match from {
            Complement => Self::Complement,
        }
    }
}

/// Predicates for matching strings
#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl From<StringPredicate> for _core::StringPredicate {
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

impl From<_core::StringPredicate> for StringPredicate {
    fn from(from: _core::StringPredicate) -> Self {
        use _core::StringPredicate::*;
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

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WithTokensQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<String>,
}

impl WithTokensQueryParams {
    pub fn try_with_token(&self, with_token: &str) -> bool {
        match self.with {
            Some(ref with) => with.split(',').any(|token| token == with_token),
            None => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<StringPredicate>,
}

impl From<StringFilter> for _core::StringFilter {
    fn from(from: StringFilter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            value: from.value.map(Into::into),
        }
    }
}

impl From<_core::StringFilter> for StringFilter {
    fn from(from: _core::StringFilter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            value: from.value.map(Into::into),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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

pub type NumericPredicate = ScalarPredicate<_core::NumericValue>;

impl From<NumericPredicate> for _core::NumericPredicate {
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

impl From<_core::NumericPredicate> for NumericPredicate {
    fn from(from: _core::NumericPredicate) -> Self {
        use _core::ScalarPredicate::*;
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

impl From<DateTimePredicate> for _core::DateTimePredicate {
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

impl From<_core::DateTimePredicate> for DateTimePredicate {
    fn from(from: _core::DateTimePredicate) -> Self {
        use _core::ScalarPredicate::*;
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ScalarFieldFilter<F, V>(pub(crate) F, pub(crate) ScalarPredicate<V>);
