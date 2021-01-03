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

#![deny(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

use aoide_core::entity::EntityHeader;

use anyhow::bail;

use std::fmt;

pub mod collection;
pub mod entity;
pub mod playlist;
pub mod tag;
pub mod track;
pub mod util;

pub type RepoId = i64;

pub type RepoError = anyhow::Error;

pub type RepoResult<T> = Result<T, RepoError>;

pub type PaginationOffset = u64;

pub type PaginationLimit = u64;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Pagination {
    pub offset: Option<PaginationOffset>,
    pub limit: Option<PaginationLimit>,
}

impl Pagination {
    pub fn none() -> Self {
        Pagination {
            offset: None,
            limit: None,
        }
    }

    pub fn is_none(&self) -> bool {
        self == &Self::none()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FilterModifier {
    Complement,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StringCompare {
    StartsWith, // head (case-insensitive)
    EndsWith,   // tail (case-insensitive)
    Contains,   // part (case-insensitive)
    Matches,    // all (case-insensitive)
    Equals,     // all (case-sensitive)
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
    Equals(String),
    EqualsNot(String),
}

impl<'a> From<&'a StringPredicate> for (StringCompare, &'a String, bool) {
    fn from(from: &'a StringPredicate) -> (StringCompare, &'a String, bool) {
        match from {
            StringPredicate::StartsWith(s) => (StringCompare::StartsWith, s, true),
            StringPredicate::StartsNotWith(s) => (StringCompare::StartsWith, s, false),
            StringPredicate::EndsWith(s) => (StringCompare::EndsWith, s, true),
            StringPredicate::EndsNotWith(s) => (StringCompare::EndsWith, s, false),
            StringPredicate::Contains(s) => (StringCompare::Contains, s, true),
            StringPredicate::ContainsNot(s) => (StringCompare::Contains, s, false),
            StringPredicate::Matches(s) => (StringCompare::Matches, s, true),
            StringPredicate::MatchesNot(s) => (StringCompare::Matches, s, false),
            StringPredicate::Equals(s) => (StringCompare::Equals, s, true),
            StringPredicate::EqualsNot(s) => (StringCompare::Equals, s, false),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StringFilter {
    pub modifier: Option<FilterModifier>,
    pub value: Option<StringPredicate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringCount {
    pub value: Option<String>,
    pub total_count: usize,
}

pub type NumericValue = f64;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NumericPredicate {
    LessThan(NumericValue),
    LessOrEqual(NumericValue),
    GreaterThan(NumericValue),
    GreaterOrEqual(NumericValue),
    Equal(Option<NumericValue>),
    NotEqual(Option<NumericValue>),
}
