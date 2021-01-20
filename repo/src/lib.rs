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

#[macro_use]
mod macros;

pub mod collection;
pub mod media;
pub mod playlist;
pub mod tag;
pub mod track;

use aoide_core::util::clock::DateTime;

pub type RecordId = i64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordHeader<Id> {
    pub id: Id,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

pub mod prelude {
    use aoide_core::util::clock::DateTime;
    use thiserror::Error;

    pub trait RecordCollector {
        type Header;
        type Record;

        /// Collect a new element
        fn collect(&mut self, header: Self::Header, record: Self::Record);
    }

    pub trait ReservableRecordCollector: RecordCollector {
        /// Reserve additional capacity for new elements
        fn reserve(&mut self, additional: usize);
    }

    pub type PaginationOffset = u64;

    pub type PaginationLimit = u64;

    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub struct Pagination {
        pub limit: PaginationLimit,
        pub offset: Option<PaginationOffset>,
    }

    #[derive(Error, Debug)]
    pub enum RepoError {
        #[error("not found")]
        NotFound,

        #[error("conflict")]
        Conflict,

        #[error(transparent)]
        Other(#[from] anyhow::Error),
    }

    pub type RepoResult<T> = Result<T, RepoError>;

    pub trait OptionalRepoResult<T> {
        fn optional(self) -> RepoResult<Option<T>>;
    }

    impl<T> OptionalRepoResult<T> for Result<T, RepoError> {
        fn optional(self) -> RepoResult<Option<T>> {
            self.map_or_else(
                |err| {
                    if matches!(err, RepoError::NotFound) {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                },
                |val| Ok(Some(val)),
            )
        }
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum SortDirection {
        Ascending,
        Descending,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum FilterModifier {
        Complement,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum StringCompare {
        StartsWith,   // head (case-insensitive)
        EndsWith,     // tail (case-insensitive)
        Contains,     // part (case-insensitive)
        Matches,      // all (case-insensitive)
        Prefix, // head (case-sensitive)
        Equals,       // all (case-sensitive)
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
        pub fn borrow(&self) -> StringFilterBorrowed<'_> {
            let Self { modifier, value } = self;
            StringFilterBorrowed {
                modifier: *modifier,
                value: value.as_ref().map(StringPredicate::borrow),
            }
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct StringCount {
        pub value: Option<String>,
        pub total_count: usize,
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

    pub type NumericValue = f64;

    pub type NumericPredicate = ScalarPredicate<NumericValue>;

    pub type DateTimePredicate = ScalarPredicate<DateTime>;
}
