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
pub enum CacheStatus {
    Current = 0,
    Outdated = 1,
    Updated = 2,
    Orphaned = 3,
}

pub type CacheDigest = DigestBytes;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub uri: String,
    pub status: CacheStatus,
    pub digest: CacheDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOutcome {
    Current,
    Inserted,
    Updated,
}

impl UpdateOutcome {
    pub const fn resulting_status(self) -> CacheStatus {
        match self {
            Self::Current => CacheStatus::Current,
            Self::Inserted => CacheStatus::Updated,
            Self::Updated => CacheStatus::Updated,
        }
    }
}

impl From<UpdateOutcome> for CacheStatus {
    fn from(from: UpdateOutcome) -> Self {
        from.resulting_status()
    }
}

pub trait Repo {
    fn media_dir_cache_update_entries_status(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri_prefix: &str,
        old_status: Option<CacheStatus>,
        new_status: CacheStatus,
    ) -> RepoResult<usize>;

    fn media_dir_cache_update_entry_digest(
        &self,
        updated_at: DateTime,
        collection_id: CollectionId,
        uri: &str,
        digest: &CacheDigest,
    ) -> RepoResult<UpdateOutcome>;

    fn media_dir_cache_delete_entries(
        &self,
        collection_id: CollectionId,
        uri_prefix: &str,
        old_status: Option<CacheStatus>,
    ) -> RepoResult<usize>;
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
