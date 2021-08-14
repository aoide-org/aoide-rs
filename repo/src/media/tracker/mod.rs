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

use crate::{
    collection::RecordId as CollectionId, media::source::RecordId as MediaSourceId, prelude::*,
};

use aoide_core::{media::SourcePath, usecases::media::tracker::DirectoriesStatus, util::clock::*};

use num_derive::{FromPrimitive, ToPrimitive};

record_id_newtype!(RecordId);
pub type RecordHeader = crate::RecordHeader<RecordId>;

record_id_newtype!(DirCacheRecordId);
pub type DirCacheRecordHeader = crate::RecordHeader<DirCacheRecordId>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, FromPrimitive, ToPrimitive)]
pub enum DirTrackingStatus {
    Current = 0,
    Outdated = 1,
    Added = 2,
    Modified = 3,
    Orphaned = 4,
}

impl DirTrackingStatus {
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
pub struct TrackedDirectory {
    pub path: SourcePath,
    pub status: DirTrackingStatus,
    pub digest: DigestBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirUpdateOutcome {
    Current,
    Inserted,
    Updated,
    Skipped,
}

impl DirUpdateOutcome {
    pub const fn resulting_status(self) -> DirTrackingStatus {
        match self {
            Self::Current => DirTrackingStatus::Current,
            Self::Inserted => DirTrackingStatus::Added,
            Self::Updated => DirTrackingStatus::Modified,
            Self::Skipped => DirTrackingStatus::Outdated,
        }
    }
}

impl From<DirUpdateOutcome> for DirTrackingStatus {
    fn from(from: DirUpdateOutcome) -> Self {
        from.resulting_status()
    }
}

pub trait Repo {
    fn media_tracker_update_directories_status(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
        old_status: Option<DirTrackingStatus>,
        new_status: DirTrackingStatus,
    ) -> RepoResult<usize>;

    fn media_tracker_update_directory_digest(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        path: &SourcePath,
        digest: &DigestBytes,
    ) -> RepoResult<DirUpdateOutcome>;

    fn media_tracker_untrack(
        &self,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
        status: Option<DirTrackingStatus>,
    ) -> RepoResult<usize>;

    /// Drop all existing references of old_source_id and replace
    /// them with new_source_id, i.e. new_source_id disappears and
    /// old_source_id takes over.
    fn media_tracker_relink_source(
        &self,
        old_source_id: MediaSourceId,
        new_source_id: MediaSourceId,
    ) -> RepoResult<bool>;

    fn media_tracker_purge_orphaned_directories(
        &self,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
    ) -> RepoResult<usize> {
        self.media_tracker_untrack(
            collection_id,
            path_prefix,
            Some(DirTrackingStatus::Orphaned),
        )
    }

    /// Mark all current entries as outdated before starting
    /// a directory traversal with calculating new digests.
    fn media_tracker_mark_current_directories_outdated(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
    ) -> RepoResult<usize> {
        self.media_tracker_update_directories_status(
            updated_at,
            collection_id,
            path_prefix,
            Some(DirTrackingStatus::Current),
            DirTrackingStatus::Outdated,
        )
    }

    /// Mark all outdated entries that have not been visited
    /// as orphaned.
    fn media_tracker_mark_outdated_directories_orphaned(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
    ) -> RepoResult<usize> {
        self.media_tracker_update_directories_status(
            updated_at,
            collection_id,
            path_prefix,
            Some(DirTrackingStatus::Outdated),
            DirTrackingStatus::Orphaned,
        )
    }

    /// Load pending entries
    ///
    /// Load pending entries, oldest first. Optionally entries can be
    /// filtered by URI prefix.
    fn media_tracker_load_directories_requiring_confirmation(
        &self,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
        pagination: &Pagination,
    ) -> RepoResult<Vec<TrackedDirectory>>;

    /// Confirm an entry as current.
    ///
    /// The digest may have changed in the meantime. If the given digest
    /// doesn't match the current digest then the operation does nothing.
    ///
    /// Returns true if the entry has been confirmed and is now considered
    /// current. Returns false if the operation has been rejected.
    fn media_tracker_confirm_directory(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        path: &SourcePath,
        digest: &DigestBytes,
        media_source_ids: &[MediaSourceId],
    ) -> RepoResult<bool>;

    fn media_tracker_load_directory_tracking_status(
        &self,
        collection_id: CollectionId,
        path: &SourcePath,
    ) -> RepoResult<DirTrackingStatus>;

    fn media_tracker_aggregate_directories_tracking_status(
        &self,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
    ) -> RepoResult<DirectoriesStatus>;

    fn media_tracker_find_untracked_sources(
        &self,
        collection_id: CollectionId,
        path_prefix: &SourcePath,
    ) -> RepoResult<Vec<MediaSourceId>>;
}
