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

use aoide_core::{media::SourcePath, util::clock::DateTime};

use aoide_core_api::media::tracker::{DirTrackingStatus, DirectoriesStatus};

use crate::{
    collection::RecordId as CollectionId, media::source::RecordId as MediaSourceId, prelude::*,
};

use super::*;

record_id_newtype!(RecordId);
pub type RecordHeader = crate::RecordHeader<RecordId>;

record_id_newtype!(DirCacheRecordId);
pub type DirCacheRecordHeader = crate::RecordHeader<DirCacheRecordId>;

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
    #[must_use]
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

    fn media_tracker_untrack_directories(
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
        self.media_tracker_untrack_directories(
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

    fn media_tracker_replace_directory_sources(
        &self,
        collection_id: CollectionId,
        path: &SourcePath,
        media_source_ids: &[MediaSourceId],
    ) -> RepoResult<(usize, usize)>;

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
        directory_path: &SourcePath,
        digest: &DigestBytes,
    ) -> RepoResult<bool>;

    fn media_tracker_load_directory_tracking_status(
        &self,
        collection_id: CollectionId,
        directory_path: &SourcePath,
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

    fn media_tracker_resolve_source_id_synchronized_at_by_path(
        &self,
        collection_id: CollectionId,
        path: &SourcePath,
    ) -> RepoResult<(MediaSourceId, Option<DateTime>)>;
}
