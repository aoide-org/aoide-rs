// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::util::clock::DateTime;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FilterModifier {
    Complement,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StringCompare {
    StartsWith, // head (case-insensitive)
    EndsWith,   // tail (case-insensitive)
    Contains,   // part (case-insensitive)
    Matches,    // all (case-insensitive)
    Prefix,     // head (case-sensitive)
    Equals,     // all (case-sensitive)
}

/// Predicates for matching strings
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StringPredicateBorrowed<'s> {
    // Case-sensitive comparison
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StringPredicate {
    // Case-sensitive comparison
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

impl<'s> From<StringPredicateBorrowed<'s>> for (StringCompare, &'s str, bool) {
    fn from(from: StringPredicateBorrowed<'s>) -> (StringCompare, &'s str, bool) {
        use StringPredicateBorrowed::*;
        match from {
            StartsWith(s) => (StringCompare::StartsWith, s, true),
            StartsNotWith(s) => (StringCompare::StartsWith, s, false),
            EndsWith(s) => (StringCompare::EndsWith, s, true),
            EndsNotWith(s) => (StringCompare::EndsWith, s, false),
            Contains(s) => (StringCompare::Contains, s, true),
            ContainsNot(s) => (StringCompare::Contains, s, false),
            Matches(s) => (StringCompare::Matches, s, true),
            MatchesNot(s) => (StringCompare::Matches, s, false),
            Prefix(s) => (StringCompare::Prefix, s, true),
            Equals(s) => (StringCompare::Equals, s, true),
            EqualsNot(s) => (StringCompare::Equals, s, false),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StringFilterBorrowed<'s> {
    pub modifier: Option<FilterModifier>,
    pub value: Option<StringPredicateBorrowed<'s>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScalarPredicate<V> {
    LessThan(V),
    LessOrEqual(V),
    GreaterThan(V),
    GreaterOrEqual(V),
    Equal(Option<V>),    // nullable
    NotEqual(Option<V>), // nullable
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ScalarFieldFilter<F, V> {
    pub field: F,
    pub predicate: ScalarPredicate<V>,
}

pub type NumericValue = f64;

pub type NumericPredicate = ScalarPredicate<NumericValue>;

pub type DateTimePredicate = ScalarPredicate<DateTime>;
