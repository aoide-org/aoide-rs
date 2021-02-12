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
pub enum TrackingStatus {
    Current = 0,
    Outdated = 1,
    Added = 2,
    Modified = 3,
    Orphaned = 4,
}

impl TrackingStatus {
    /// Determine if an entry is stale.
    pub fn is_stale(self) -> bool {
        match self {
            Self::Outdated | Self::Added | Self::Modified => true,
            Self::Current | Self::Orphaned => false,
        }
    }

    /// Determine if an entry is stale and requires further processing.
    pub fn is_pending(self) -> bool {
        match self {
            Self::Added | Self::Modified => {
                debug_assert!(self.is_stale());
                true
            }
            Self::Current | Self::Outdated | Self::Orphaned => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub uri: String,
    pub status: TrackingStatus,
    pub digest: DigestBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOutcome {
    Current,
    Inserted,
    Updated,
    Skipped,
}

impl UpdateOutcome {
    pub const fn resulting_status(self) -> TrackingStatus {
        match self {
            Self::Current => TrackingStatus::Current,
            Self::Inserted => TrackingStatus::Added,
            Self::Updated => TrackingStatus::Modified,
            Self::Skipped => TrackingStatus::Outdated,
        }
    }
}

impl From<UpdateOutcome> for TrackingStatus {
    fn from(from: UpdateOutcome) -> Self {
        from.resulting_status()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TrackingStatusAggregated {
    pub current: usize,
    pub outdated: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
}

pub trait Repo {
    fn media_dir_tracker_update_entries_status(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri_prefix: &str,
        old_status: Option<TrackingStatus>,
        new_status: TrackingStatus,
    ) -> RepoResult<usize>;

    fn media_dir_tracker_update_entry_digest(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri: &str,
        digest: &DigestBytes,
    ) -> RepoResult<UpdateOutcome>;

    fn media_dir_tracker_delete_entries(
        &self,
        collection_id: CollectionId,
        uri_prefix: &str,
        old_status: Option<TrackingStatus>,
    ) -> RepoResult<usize>;

    /// Mark all current entries as outdated before starting
    /// a directory traversal with calculating new digests.
    fn media_dir_tracker_mark_entries_outdated(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri_prefix: &str,
    ) -> RepoResult<usize> {
        self.media_dir_tracker_update_entries_status(
            updated_at,
            collection_id,
            uri_prefix,
            Some(TrackingStatus::Current),
            TrackingStatus::Outdated,
        )
    }

    /// Mark all outdated entries that have not been visited
    /// as orphaned.
    fn media_dir_tracker_mark_entries_orphaned(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri_prefix: &str,
    ) -> RepoResult<usize> {
        self.media_dir_tracker_update_entries_status(
            updated_at,
            collection_id,
            uri_prefix,
            Some(TrackingStatus::Outdated),
            TrackingStatus::Orphaned,
        )
    }

    /// Confirm an entry as current.
    ///
    /// The digest may have changed in the meantime. If the given digest
    /// doesn't match the current digest then the operation does nothing.
    ///
    /// Returns true if the entry has been confirmed and is now considered
    /// current. Returns false if the operation has been rejected.
    fn media_dir_tracker_confirm_entry_digest_current(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri: &str,
        digest: &DigestBytes,
    ) -> RepoResult<bool>;

    fn media_dir_tracker_load_entry_status(
        &self,
        collection_id: CollectionId,
        uri: &str,
    ) -> RepoResult<TrackingStatus>;

    fn media_dir_tracker_update_load_aggregate_status(
        &self,
        collection_id: CollectionId,
        uri_prefix: &str,
    ) -> RepoResult<TrackingStatusAggregated>;

    /// Load pending entries
    ///
    /// Load pending entries, oldest first. Optionally entries can be
    /// filtered by URI prefix.
    fn media_dir_tracker_load_pending_entries(
        &self,
        collection_id: CollectionId,
        uri_prefix: Option<&str>,
        pagination: &Pagination,
    ) -> RepoResult<Vec<Entry>>;
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
