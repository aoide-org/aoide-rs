// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![warn(clippy::clone_on_ref_ptr)]
// Repetitions of module/type names occur frequently when using many
// modules for keeping the size of the source files handy. Often
// types have the same name as their parent module.
#![allow(clippy::module_name_repetitions)]
// Repeating the type name in `..Default::default()` expressions
// is not needed since the context is obvious.
#![allow(clippy::default_trait_access)]
// Using wildcard imports consciously is acceptable.
#![allow(clippy::wildcard_imports)]
// Importing all enum variants into a narrow, local scope is acceptable.
#![allow(clippy::enum_glob_use)]
// TODO: Add missing docs
#![allow(clippy::missing_errors_doc)]

#[macro_use]
mod macros;

pub mod collection;
pub mod media;
pub mod playlist;
pub mod tag;
pub mod track;

use aoide_core::util::clock::DateTime;

pub type RecordId = i64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordHeader<Id> {
    pub id: Id,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

pub mod prelude {
    pub use aoide_core_api::{
        filtering::*, sorting::*, Pagination, PaginationLimit, PaginationOffset,
    };
    use thiserror::Error;

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

    pub fn fetch_and_collect_filtered_records<R, T, Header, Record, Fetch, FilterMap, Collector>(
        repo: &mut R,
        pagination: Option<&Pagination>,
        mut fetch: Fetch,
        mut filter_map: FilterMap,
        collector: &mut Collector,
    ) -> RepoResult<()>
    where
        Fetch: FnMut(&mut R, Option<&Pagination>) -> RepoResult<Vec<T>>,
        FilterMap: FnMut(&mut R, T) -> RepoResult<Option<(Header, Record)>>,
        Collector: ReservableRecordCollector<Header = Header, Record = Record> + ?Sized,
    {
        let mut pagination = pagination.cloned();
        loop {
            let fetched_records = fetch(repo, pagination.as_ref())?;
            if fetched_records.is_empty() {
                break;
            }
            collector.reserve(fetched_records.len());
            let num_fetched_records = fetched_records.len() as PaginationOffset;
            let mut num_discarded_records = 0usize;
            for record in fetched_records {
                if let Some((header, record)) = filter_map(repo, record)? {
                    collector.collect(header, record);
                } else {
                    num_discarded_records += 1;
                }
            }
            if num_discarded_records == 0 {
                break;
            }
            if let Some(pagination) = &mut pagination {
                if let Some(limit) = &mut pagination.limit {
                    debug_assert!(num_fetched_records <= *limit);
                    if num_fetched_records >= *limit {
                        break;
                    }
                    // Fetch remaining records according to pagination with
                    // one or more subsequent queries.
                    pagination.offset = Some(pagination.offset.unwrap_or(0) + num_fetched_records);
                    *limit -= num_fetched_records;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(())
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

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct StringCount {
        pub value: Option<String>,
        pub total_count: usize,
    }
}
