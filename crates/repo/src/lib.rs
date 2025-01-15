// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use derive_more::derive::{Display, Error};

use aoide_core::util::clock::OffsetDateTimeMs;
use aoide_core_api::{Pagination, PaginationOffset};

#[macro_use]
mod macros;

pub mod collection;
pub use self::collection::RecordId as CollectionId;

pub mod media;
pub use self::media::source::RecordId as MediaSourceId;

pub mod playlist;
pub use self::playlist::RecordId as PlaylistId;

pub mod tag;

pub mod track;
pub use self::track::RecordId as TrackId;

pub type RecordId = i64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordHeader<Id> {
    pub id: Id,
    pub created_at: OffsetDateTimeMs,
    pub updated_at: OffsetDateTimeMs,
}

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

#[derive(Debug, Display, Error)]
pub enum RepoError {
    #[display("not found")]
    NotFound,

    #[display("conflict")]
    Conflict,

    #[display("aborted")]
    Aborted,

    Other(anyhow::Error),
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
