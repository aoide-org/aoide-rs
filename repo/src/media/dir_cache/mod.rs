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

use super::*;

use crate::{collection::RecordId as CollectionId, prelude::*};

use aoide_core::util::clock::*;

use num_derive::{FromPrimitive, ToPrimitive};

record_id_newtype!(RecordId);
pub type RecordHeader = crate::RecordHeader<RecordId>;

record_id_newtype!(DirCacheRecordId);
pub type DirCacheRecordHeader = crate::RecordHeader<DirCacheRecordId>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
pub enum EntryStatus {
    Current = 0,
    Outdated = 1,
    Added = 2,
    Modified = 3,
    Orphaned = 4,
}

pub type EntryDigest = DigestBytes;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub uri: String,
    pub status: EntryStatus,
    pub digest: EntryDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOutcome {
    Current,
    Inserted,
    Updated,
    Skipped,
}

impl UpdateOutcome {
    pub const fn resulting_status(self) -> EntryStatus {
        match self {
            Self::Current => EntryStatus::Current,
            Self::Inserted => EntryStatus::Added,
            Self::Updated => EntryStatus::Modified,
            Self::Skipped => EntryStatus::Outdated,
        }
    }
}

impl From<UpdateOutcome> for EntryStatus {
    fn from(from: UpdateOutcome) -> Self {
        from.resulting_status()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AggregateStatus {
    pub current: usize,
    pub outdated: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
}

pub trait Repo {
    fn media_dir_cache_update_entries_status(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri_prefix: &str,
        old_status: Option<EntryStatus>,
        new_status: EntryStatus,
    ) -> RepoResult<usize>;

    fn media_dir_cache_update_entry_digest(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri: &str,
        digest: &EntryDigest,
    ) -> RepoResult<UpdateOutcome>;

    fn media_dir_cache_delete_entries(
        &self,
        collection_id: CollectionId,
        uri_prefix: &str,
        old_status: Option<EntryStatus>,
    ) -> RepoResult<usize>;

    /// Mark all current entries as outdated before starting
    /// a directory traversal with calculating new digests.
    fn media_dir_cache_mark_entries_outdated(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri_prefix: &str,
    ) -> RepoResult<usize> {
        self.media_dir_cache_update_entries_status(
            updated_at,
            collection_id,
            uri_prefix,
            Some(EntryStatus::Current),
            EntryStatus::Outdated,
        )
    }

    /// Mark all outdated entries that have not been visited
    /// as orphaned.
    fn media_dir_cache_mark_entries_orphaned(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri_prefix: &str,
    ) -> RepoResult<usize> {
        self.media_dir_cache_update_entries_status(
            updated_at,
            collection_id,
            uri_prefix,
            Some(EntryStatus::Outdated),
            EntryStatus::Orphaned,
        )
    }

    fn media_dir_cache_reset_entry_status_to_current(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri: &str,
        digest: &EntryDigest,
    ) -> RepoResult<bool>;

    fn media_dir_cache_load_entry_status_by_uri(
        &self,
        collection_id: CollectionId,
        uri: &str,
    ) -> RepoResult<EntryStatus>;

    fn media_dir_cache_update_load_entries_aggregate_status(
        &self,
        collection_id: CollectionId,
        uri_prefix: &str,
    ) -> RepoResult<AggregateStatus>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaSourceSummary {
    pub total_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackSummary {
    pub total_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaylistSummary {
    pub total_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Summary {
    pub media_sources: Option<MediaSourceSummary>,
    pub tracks: Option<TrackSummary>,
    pub playlists: Option<PlaylistSummary>,
}
