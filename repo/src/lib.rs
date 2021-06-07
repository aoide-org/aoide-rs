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
    use thiserror::Error;

    pub use aoide_core::usecases::{filtering::*, sorting::*};

    pub trait RecordCollector {
        type Header;
        type Record;

        /// Collect a new element
        fn collect(&mut self, header: Self::Header, record: Self::Record);
    }

    impl<H, R> RecordCollector for Vec<(H, R)> {
        type Header = H;
        type Record = R;

        fn collect(&mut self, header: Self::Header, record: Self::Record) {
            self.push((header, record));
        }
    }

    pub trait ReservableRecordCollector: RecordCollector {
        /// Reserve additional capacity for new elements
        fn reserve(&mut self, additional: usize);
    }

    impl<H, R> ReservableRecordCollector for Vec<(H, R)> {
        fn reserve(&mut self, additional: usize) {
            Vec::reserve(self, additional);
        }
    }

    pub type PaginationOffset = u64;

    pub type PaginationLimit = u64;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct Pagination {
        pub limit: PaginationLimit,
        pub offset: Option<PaginationOffset>,
    }

    impl Default for Pagination {
        fn default() -> Self {
            Self {
                limit: PaginationLimit::max_value(),
                offset: None,
            }
        }
    }

    #[derive(Error, Debug)]
    pub enum RepoError {
        #[error("not found")]
        NotFound,

        #[error("conflict")]
        Conflict,

        #[error("aborted")]
        Aborted,

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

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct StringCount {
        pub value: Option<String>,
        pub total_count: usize,
    }
}
